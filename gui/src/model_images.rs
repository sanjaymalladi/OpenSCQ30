//! Selected-model product imagery loaded from a URL at runtime.
//!
//! Privacy: fetching contacts only the configured static image host. No device
//! identifiers, addresses, serial numbers, firmware values, or settings are
//! ever sent with the request.
//!
//! Catalog entries carry a URL only when the direct image URL is verified. A
//! missing URL is a valid state and shows the neutral fallback. Do not
//! fabricate URLs or infer CDN paths. Manufacturer/CDN hotlinks may change or
//! break; image-use rights must be confirmed before shipping any URL.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use cosmic::{
    Element,
    iced::{
        Length,
        widget::image::{Handle, Image},
    },
    widget,
};
use openscq30_lib::DeviceModel;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, oneshot};

/// Maximum accepted image download size (8 MiB).
const MAX_IMAGE_BYTES: u64 = 8 * 1024 * 1024;
/// Network timeout for a single image fetch.
const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
/// Cached images older than this are refreshed in the background.
const STALE_AFTER_SECS: u64 = 30 * 24 * 60 * 60;

/// Validated image file extensions for cache files.
const IMAGE_EXTENSIONS: [&str; 5] = ["png", "jpg", "webp", "gif", "bin"];

/// One entry of the model image catalog.
pub struct CatalogEntry {
    /// Canonical product code, also used for cache file names.
    pub code: &'static str,
    /// Verified direct HTTPS image URL, if one is known.
    pub url: Option<&'static str>,
    /// Source/attribution note for the URL above.
    pub attribution: &'static str,
}

/// Canonical product code for a device model. Variants sharing one device
/// implementation share one canonical identity.
pub fn canonical_code(model: DeviceModel) -> &'static str {
    use DeviceModel as M;
    match model {
        M::SoundcoreA3004 => "A3004",
        M::SoundcoreA3005 => "A3005",
        M::SoundcoreA3027 | M::SoundcoreA3030 => "A3027",
        M::SoundcoreA3028 | M::SoundcoreA3029 => "A3028",
        M::SoundcoreA3031 => "A3031",
        M::SoundcoreA3033 => "A3033",
        M::SoundcoreA3035 => "A3035",
        M::SoundcoreA3040 => "A3040",
        M::SoundcoreA3062 => "A3062",
        M::SoundcoreA3116 => "A3116",
        M::SoundcoreA3330 => "A3330",
        M::SoundcoreA3388 => "A3388",
        M::SoundcoreA3876 => "A3876",
        M::SoundcoreA3909 => "A3909",
        M::SoundcoreA3926 => "A3926",
        M::SoundcoreA3930 => "A3930",
        M::SoundcoreA3931 | M::SoundcoreA3935 => "A3931",
        M::SoundcoreA3933 | M::SoundcoreA3939 => "A3933",
        M::SoundcoreA3936 => "A3936",
        M::SoundcoreA3944 => "A3944",
        M::SoundcoreA3945 => "A3945",
        M::SoundcoreA3951 => "A3951",
        M::SoundcoreA3952 => "A3952",
        M::SoundcoreA3953 => "A3953",
        M::SoundcoreA3954 => "A3954",
        M::SoundcoreA3955 => "A3955",
        M::SoundcoreA3957 => "A3957",
        M::SoundcoreA3959 => "A3959",
        M::SoundcoreA3968 => "A3968",
        M::SoundcoreA3947 => "A3947",
        M::SoundcoreA3948 => "A3948",
        M::SoundcoreA3949 => "A3949",
        M::SoundcoreD1101 => "D1101",
        M::SoundcoreD1202 | M::SoundcoreD1202C => "D1202",
        M::SoundcoreD1301 => "D1301",
        M::SoundcoreA6611 => "A6611",
        M::SoundcoreDevelopment => "DEVELOPMENT",
    }
}

/// Catalog entry for a device model.
pub fn catalog_entry(model: DeviceModel) -> CatalogEntry {
    let code = canonical_code(model);
    // Only A3004 currently has a verified direct image URL (official Soundcore
    // CDN `og:image` for the Q20i, checked 2026-10-13; PNG, ~2 MB, dark
    // product shot that suits the dark UI). Shopify `v=` query strings can be
    // rotated by the host; treat fetch failures as missing and fall back.
    // Usage rights for shipping this image in the app are NOT confirmed.
    let (url, attribution) = match model {
        DeviceModel::SoundcoreA3004 => (
            Some(
                "https://cdn.shopify.com/s/files/1/0516/3761/6830/files/20251230-142814.png?v=1767076187",
            ),
            "Image: soundcore (soundcore.com) Q20i product render, model identification only; usage rights unconfirmed",
        ),
        _ => (None, "No verified image URL; using neutral fallback"),
    };
    CatalogEntry {
        code,
        url,
        attribution,
    }
}

/// Only static HTTPS resources may be fetched.
fn is_fetchable_url(url: &str) -> bool {
    url.starts_with("https://") && url.len() > "https://".len() + 1
}

#[derive(Serialize, Deserialize)]
struct CacheMeta {
    url: String,
    extension: String,
    fetched_at_unix: u64,
}

fn cache_stem(cache_dir: &std::path::Path, code: &str) -> PathBuf {
    cache_dir.join(code)
}

fn decode_handle(bytes: &[u8]) -> Option<Handle> {
    let image = image::load_from_memory(bytes).ok()?;
    let mut rgba = image.to_rgba8();
    // Remove the faint white bloom baked into some product renders. Keep the
    // opaque headphone highlights; only fully white, translucent fringe is
    // cleared so it blends into both light and dark app themes.
    for pixel in rgba.pixels_mut() {
        let [red, green, blue, alpha] = pixel.0;
        let brightest = red.max(green).max(blue);
        let darkest = red.min(green).min(blue);
        if alpha < 190 && red > 220 && green > 220 && blue > 220 && brightest - darkest < 20 {
            pixel.0 = [0, 0, 0, 0];
        }
    }
    let (width, height) = (rgba.width(), rgba.height());
    if width == 0 || height == 0 {
        return None;
    }
    Some(Handle::from_rgba(
        width,
        height,
        bytes::Bytes::from(rgba.into_raw()),
    ))
}

struct Cached {
    bytes: Vec<u8>,
    stale: bool,
}

enum LoadState {
    Loading(Vec<oneshot::Sender<Option<Handle>>>),
    Ready(Option<Handle>),
}

struct ServiceInner {
    client: reqwest::Client,
    cache_dir: PathBuf,
    state: Mutex<HashMap<String, LoadState>>,
}

/// Async selected-model image service with memory + disk cache and fallback.
///
/// Cloneable; concurrent loads for the same model are deduplicated. Late
/// responses must be guarded by the caller against the current selection.
#[derive(Clone)]
pub struct ModelImageService {
    inner: Arc<ServiceInner>,
}

impl ModelImageService {
    /// Creates the service. `cache_dir` should be the platform cache dir
    /// joined with `openscq30/model-images`. Directory creation failures are
    /// tolerated (memory-only operation).
    pub fn new(cache_dir: PathBuf) -> Self {
        let client = reqwest::Client::builder().timeout(FETCH_TIMEOUT).build();
        let client = client.unwrap_or_else(|_| reqwest::Client::new());
        Self {
            inner: Arc::new(ServiceInner {
                client,
                cache_dir,
                state: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Loads the image for `model`, returning `None` for missing URLs or any
    /// URL/network/decode/cache error. Never blocks the GUI thread beyond
    /// awaiting this future from a `Task`.
    pub async fn load(&self, model: DeviceModel) -> Option<Handle> {
        let entry = catalog_entry(model);
        let key = entry.code.to_owned();

        let waiter = {
            let mut state = self.inner.state.lock().await;
            match state.get_mut(&key) {
                Some(LoadState::Ready(handle)) => return handle.clone(),
                Some(LoadState::Loading(waiters)) => {
                    let (tx, rx) = oneshot::channel();
                    waiters.push(tx);
                    Some(rx)
                }
                None => {
                    state.insert(key.clone(), LoadState::Loading(Vec::new()));
                    None
                }
            }
        };
        if let Some(rx) = waiter {
            return rx.await.unwrap_or(None);
        }

        let result = self.fetch_for_entry(&entry).await;

        let mut state = self.inner.state.lock().await;
        if let Some(LoadState::Loading(waiters)) =
            state.insert(key, LoadState::Ready(result.clone()))
        {
            for waiter in waiters {
                let _ = waiter.send(result.clone());
            }
        }
        result
    }

    async fn fetch_for_entry(&self, entry: &CatalogEntry) -> Option<Handle> {
        let url = entry.url.filter(|url| is_fetchable_url(url))?;
        let stem = self.inner.cache_dir.join(entry.code);

        // Disk cache first (also reused when offline).
        if let Some(cached) = self.load_disk_cache(&stem, url).await {
            // Refresh stale entries in the background without delaying the UI.
            if cached.stale {
                let this = self.clone();
                let url = url.to_owned();
                tokio::spawn(async move {
                    this.refresh_disk_cache(&stem, &url).await;
                });
            }
            return decode_handle(&cached.bytes);
        }

        let bytes = self.download(url).await?;
        self.store_disk_cache(&stem, url, &bytes).await;
        decode_handle(&bytes)
    }

    async fn load_disk_cache(&self, stem: &Path, url: &str) -> Option<Cached> {
        let meta_path = stem.with_extension("meta.toml");
        let meta_bytes = tokio::fs::read(&meta_path).await.ok()?;
        let meta: CacheMeta = toml::from_str(&String::from_utf8(meta_bytes).ok()?).ok()?;
        if meta.url != url || !IMAGE_EXTENSIONS.contains(&meta.extension.as_str()) {
            return None;
        }
        let bytes = tokio::fs::read(stem.with_extension(&meta.extension))
            .await
            .ok()?;
        if bytes.is_empty() || bytes.len() as u64 > MAX_IMAGE_BYTES {
            return None;
        }
        let now = now_unix().unwrap_or(meta.fetched_at_unix);
        Some(Cached {
            bytes,
            stale: now.saturating_sub(meta.fetched_at_unix) > STALE_AFTER_SECS,
        })
    }

    async fn refresh_disk_cache(&self, stem: &Path, url: &str) {
        if let Some(bytes) = self.download(url).await {
            self.store_disk_cache(stem, url, &bytes).await;
            if let Some(handle) = decode_handle(&bytes) {
                let key = stem
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_owned();
                let mut state = self.inner.state.lock().await;
                state.insert(key, LoadState::Ready(Some(handle)));
            }
        }
    }

    async fn download(&self, url: &str) -> Option<Vec<u8>> {
        let response = tokio::time::timeout(FETCH_TIMEOUT, self.inner.client.get(url).send())
            .await
            .ok()?
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        let mime = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        if !mime.starts_with("image/") {
            return None;
        }
        if response
            .content_length()
            .is_some_and(|len| len > MAX_IMAGE_BYTES)
        {
            return None;
        }
        let bytes = tokio::time::timeout(FETCH_TIMEOUT, response.bytes())
            .await
            .ok()?
            .ok()?;
        if bytes.is_empty() || bytes.len() as u64 > MAX_IMAGE_BYTES {
            return None;
        }
        // Re-check decodeability before caching so corrupt payloads never land
        // on disk.
        decode_handle(&bytes)?;
        Some(bytes.to_vec())
    }

    async fn store_disk_cache(&self, stem: &Path, url: &str, bytes: &[u8]) {
        if tokio::fs::create_dir_all(&self.inner.cache_dir)
            .await
            .is_err()
        {
            return;
        }
        // Extension is derived from already-validated bytes, not the URL.
        let extension = image::guess_format(bytes).map_or("bin", |format| match format {
            image::ImageFormat::Png => "png",
            image::ImageFormat::Jpeg => "jpg",
            image::ImageFormat::WebP => "webp",
            image::ImageFormat::Gif => "gif",
            _ => "bin",
        });
        let meta = CacheMeta {
            url: url.to_owned(),
            extension: extension.to_owned(),
            fetched_at_unix: now_unix().unwrap_or(0),
        };
        let meta_text = toml::to_string(&meta).unwrap_or_default();
        // Best effort: any failure just means memory-only operation.
        let _ = tokio::fs::write(stem.with_extension(extension), bytes).await;
        let _ = tokio::fs::write(stem.with_extension("meta.toml"), meta_text.as_str()).await;
        // Remove images stored under a different extension from an earlier run.
        for other in IMAGE_EXTENSIONS {
            if other != extension {
                let _ = tokio::fs::remove_file(stem.with_extension(other)).await;
            }
        }
    }
}

fn now_unix() -> Option<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .ok()
}

/// Image element for a loaded handle at the given width (aspect preserved),
/// or an empty placeholder when the model has no image. The neutral fallback
/// (icon + model name) is composed by the calling page, which owns the model
/// identity context.
pub fn model_image<'a, M>(handle: Option<&Handle>, width: f32) -> Element<'a, M>
where
    M: Clone + 'static,
{
    match handle {
        Some(handle) => Image::new(handle.clone())
            .width(Length::Fixed(width))
            .into(),
        None => widget::text("").into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openscq30_lib::DeviceModel;
    use strum::VariantArray;

    #[test]
    fn every_model_resolves_to_a_code() {
        for model in DeviceModel::VARIANTS {
            assert!(!canonical_code(*model).is_empty());
            let _ = catalog_entry(*model);
        }
    }

    #[test]
    fn aliases_share_canonical_identity() {
        use DeviceModel as M;
        assert_eq!(
            canonical_code(M::SoundcoreA3027),
            canonical_code(M::SoundcoreA3030)
        );
        assert_eq!(
            canonical_code(M::SoundcoreA3028),
            canonical_code(M::SoundcoreA3029)
        );
        assert_eq!(
            canonical_code(M::SoundcoreA3931),
            canonical_code(M::SoundcoreA3935)
        );
        assert_eq!(
            canonical_code(M::SoundcoreA3933),
            canonical_code(M::SoundcoreA3939)
        );
        assert_eq!(
            canonical_code(M::SoundcoreD1202),
            canonical_code(M::SoundcoreD1202C)
        );
    }

    #[test]
    fn only_https_urls_are_fetchable() {
        assert!(is_fetchable_url("https://cdn.example.com/a.png"));
        assert!(!is_fetchable_url("http://cdn.example.com/a.png"));
        assert!(!is_fetchable_url("ftp://cdn.example.com/a.png"));
        assert!(!is_fetchable_url("https://"));
        assert!(!is_fetchable_url(""));
        assert!(!is_fetchable_url("not a url"));
    }

    #[test]
    fn garbage_bytes_do_not_decode() {
        assert!(decode_handle(&[]).is_none());
        assert!(decode_handle(b"definitely not an image").is_none());
    }

    #[test]
    fn cache_meta_roundtrips() {
        let meta = CacheMeta {
            url: "https://cdn.example.com/a.png".to_owned(),
            extension: "png".to_owned(),
            fetched_at_unix: 1_700_000_000,
        };
        let text = toml::to_string(&meta).unwrap();
        let back: CacheMeta = toml::from_str(&text).unwrap();
        assert_eq!(back.url, meta.url);
        assert_eq!(back.extension, meta.extension);
        assert_eq!(back.fetched_at_unix, meta.fetched_at_unix);
    }

    #[test]
    fn cache_stem_joins_code() {
        let stem = cache_stem(std::path::Path::new("/tmp/cache"), "A3004");
        assert_eq!(stem, PathBuf::from("/tmp/cache/A3004"));
    }
}

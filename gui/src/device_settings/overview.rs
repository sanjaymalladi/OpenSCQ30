//! Model-aware device Overview: identity strip plus cards and shortcuts
//! assembled only from the connected device's actual capabilities.
//!
//! All values are read live through the device API; nothing here duplicates
//! setting state or bypasses the existing `SetSetting`/`SetEqualizerBand`
//! update paths. Unsupported capabilities are omitted, never faked.

use cosmic::iced::widget::image::Handle;
use cosmic::{Element, Task, iced::Length, widget};
use openscq30_i18n::Translate;
use openscq30_lib::{
    DeviceModel,
    connection::ConnectionStatus,
    settings::{CategoryId, Setting, SettingId},
};

use crate::{
    app::DebugOpenSCQ30Device,
    components::{self, SPACE_M, SPACE_S, SPACE_XS},
    equalizer_line::EqualizerLine,
    fl,
    model_images::{ModelImageService, model_image},
};

/// Navigation target for overview shortcut rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavTarget {
    Category(CategoryId),
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(NavTarget),
    ImageLoaded(DeviceModel, Option<Handle>),
}

pub struct OverviewModel {
    device: DebugOpenSCQ30Device,
    image: Option<Handle>,
}

impl OverviewModel {
    pub fn new(device: DebugOpenSCQ30Device) -> (Self, Task<Message>) {
        let cache_dir = dirs::cache_dir().map_or_else(std::env::temp_dir, |dir| {
            dir.join("openscq30").join("model-images")
        });
        let image_service = ModelImageService::new(cache_dir);
        let model = device.0.model();
        let service = image_service;
        let task =
            Task::future(async move { Message::ImageLoaded(model, service.load(model).await) });
        (
            Self {
                device,
                image: None,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) {
        if let Message::ImageLoaded(model, handle) = message
            && model == self.device.0.model()
        {
            self.image = handle;
        }
    }

    fn live(&self, setting_id: &SettingId) -> Option<Setting> {
        self.device.0.setting(setting_id)
    }

    pub fn view(&self) -> Element<'_, super::Message> {
        let mut cards = widget::row![].spacing(SPACE_M);
        let mut has_card = false;
        if let Some(card) = self.sound_modes_card() {
            cards = cards.push(card);
            has_card = true;
        }
        if let Some(card) = self.equalizer_card() {
            cards = cards.push(card);
            has_card = true;
        }
        let mut column = widget::column![self.identity()].spacing(SPACE_M);
        if has_card {
            column = column.push(cards);
        }
        column = column.push(self.device_details());
        components::page(column.into())
    }

    fn identity(&self) -> Element<'_, super::Message> {
        use crate::components::ACCENT_RED;
        let model = self.device.0.model();
        let connected = matches!(
            *self.device.0.connection_status().borrow(),
            ConnectionStatus::Connected
        );
        let status_text = if connected {
            widget::text::body(fl!("bluetooth-connected"))
        } else {
            widget::text::body(fl!("bluetooth-not-connected"))
        };
        let status = widget::row![
            widget::text("●")
                .size(12)
                .class(cosmic::theme::Text::Color(if connected {
                    ACCENT_RED
                } else {
                    crate::components::MUTED_GREY
                })),
            status_text,
        ]
        .spacing(SPACE_XS)
        .align_y(cosmic::iced::Alignment::Center);
        let mut details =
            widget::column![components::hero_title(model.translate()), status,].spacing(SPACE_S);
        if let Some(battery) = self.battery_text() {
            let (icon, color) = battery_style(parse_percents(&battery));
            let label = widget::text::body(battery);
            details = details.push(
                widget::row![
                    widget::icon::from_name(icon).size(16),
                    match color {
                        Some(color) => label.class(cosmic::theme::Text::Color(color)),
                        None => label,
                    },
                ]
                .spacing(SPACE_XS)
                .align_y(cosmic::iced::Alignment::Center),
            );
        }
        let image = match &self.image {
            Some(handle) => model_image(Some(handle), 300.0),
            None => widget::text(model.to_string()).size(16).into(),
        };
        widget::container(
            widget::row![details.width(Length::Fill), image]
                .spacing(SPACE_M)
                .align_y(cosmic::iced::Alignment::Center),
        )
        .padding([SPACE_M * 2, SPACE_M * 2])
        .width(Length::Fill)
        .into()
    }

    /// Battery summary from real level settings only. Single-battery devices
    /// show one value; dual-battery devices show both sides.
    fn battery_text(&self) -> Option<String> {
        let level = |id: SettingId| match self.live(&id)? {
            Setting::Information {
                translated_value, ..
            } => Some(translated_value),
            _ => None,
        };
        match (
            level(SettingId::BatteryLevel),
            level(SettingId::BatteryLevelLeft),
            level(SettingId::BatteryLevelRight),
        ) {
            (Some(single), _, _) => Some(format!("{}: {single}", fl!("battery"))),
            (None, Some(left), Some(right)) => {
                Some(format!("{}: {left} · {right}", fl!("battery")))
            }
            (None, Some(left), None) | (None, None, Some(left)) => {
                Some(format!("{}: {left}", fl!("battery")))
            }
            _ => None,
        }
    }

    /// Three-state ambient sound card, rendered only when the device exposes
    /// the ambient sound mode select. Options and messages come straight from
    /// the setting metadata.
    fn sound_modes_card(&self) -> Option<Element<'_, super::Message>> {
        let Setting::Select { setting, value } = self.live(&SettingId::AmbientSoundMode)? else {
            return None;
        };
        let options = setting
            .options
            .iter()
            .zip(setting.localized_options.iter())
            .enumerate()
            .map(|(index, (option, localized))| {
                (
                    localized.clone(),
                    ambient_mode_icon(localized, index),
                    option == value.as_ref(),
                    super::Message::SetSetting(SettingId::AmbientSoundMode, option.clone().into()),
                )
            })
            .collect();
        Some(components::feature_card(
            CategoryId::SoundModes.translate(),
            Some(fl!("ambient-sound-description")),
            components::mode_icon_choices(options),
        ))
    }

    /// Equalizer summary card, rendered only when the device exposes equalizer
    /// settings. Shows the current profile (or custom state), a live mini
    /// curve when band values exist, and a shortcut to the full editor.
    fn equalizer_card(&self) -> Option<Element<'_, super::Message>> {
        let Setting::PresetEqualizerProfileSelect { select, value, .. } =
            self.live(&SettingId::PresetEqualizerProfile)?
        else {
            return None;
        };
        let profile = value
            .as_deref()
            .and_then(|value| select.options.iter().position(|option| option == value))
            .and_then(|index| select.localized_options.get(index).cloned())
            .unwrap_or_else(|| SettingId::CustomEqualizerProfile.translate());
        let mut content = widget::column![].spacing(SPACE_S);
        if let Some(Setting::Equalizer { setting, value, .. }) =
            self.live(&SettingId::VolumeAdjustments)
            && !value.is_empty()
        {
            content = content.push(Element::from(
                widget::canvas(
                    EqualizerLine::new(setting.min, setting.max, value.clone())
                        .red()
                        .labels(
                            frequency_labels(&setting.band_hz),
                            db_labels(setting.min, setting.max, setting.fraction_digits),
                        ),
                )
                .width(Length::Fill)
                .height(Length::Fixed(120.0)),
            ));
        }
        content = content.push(components::status_row(
            profile,
            None::<&str>,
            Some(super::Message::Overview(Message::Navigate(
                NavTarget::Category(CategoryId::Equalizer),
            ))),
        ));
        Some(components::feature_card(
            CategoryId::Equalizer.translate(),
            None::<&str>,
            content.into(),
        ))
    }

    fn device_details(&self) -> Element<'_, super::Message> {
        let mut details = widget::row![].spacing(SPACE_M);
        details = details.push(
            widget::column![
                widget::text(fl!("device-model")).size(14),
                widget::text(crate::model_images::canonical_code(self.device.0.model())).size(16),
            ]
            .spacing(SPACE_XS)
            .width(Length::Fill),
        );
        if let Some(Setting::Information {
            translated_value, ..
        }) = self.live(&SettingId::FirmwareVersion)
        {
            details = details.push(components::vertical_divider());
            details = details.push(
                widget::column![
                    widget::text(fl!("firmware-version")).size(14),
                    widget::text(translated_value).size(16),
                ]
                .spacing(SPACE_XS)
                .width(Length::Fill),
            );
        }
        components::feature_card(fl!("device-information"), None::<&str>, details.into())
    }
}
/// Icon variant and optional tier color from reported battery percentages.
/// Unknown levels fall back to a neutral icon with default text.
fn battery_style(percents: Vec<u8>) -> (&'static str, Option<cosmic::iced::Color>) {
    use crate::components::{ACCENT_RED, BATTERY_AMBER, HEALTHY_GREEN};
    match percents.into_iter().min() {
        Some(level) if level >= 60 => ("battery-good-symbolic", Some(HEALTHY_GREEN)),
        Some(level) if level >= 25 => ("battery-caution-symbolic", Some(BATTERY_AMBER)),
        Some(_) => ("battery-low-symbolic", Some(ACCENT_RED)),
        None => ("battery-good-symbolic", None),
    }
}

/// Extracts 0–100 integers from a localized battery string.
fn parse_percents(text: &str) -> Vec<u8> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u8>().ok())
        .collect()
}

fn ambient_mode_icon(label: &str, index: usize) -> &'static str {
    let label = label.to_lowercase();
    if label.contains("transparen") || label.contains("ambient") {
        "transparency"
    } else if label.contains("cancel") || label.contains("noise") {
        "noise-canceling"
    } else {
        match index {
            1 => "transparency",
            2 => "noise-canceling",
            _ => "normal",
        }
    }
}

/// Up to five evenly spaced frequency labels from band metadata.
fn frequency_labels(band_hz: &[u16]) -> Vec<(usize, String)> {
    let count = band_hz.len();
    if count == 0 {
        return Vec::new();
    }
    let shown = count.min(5);
    (0..shown)
        .map(|k| {
            let index = if shown == 1 {
                0
            } else {
                k * (count - 1) / (shown - 1)
            };
            let hz = band_hz[index];
            let text = if hz >= 1000 {
                format!("{}K", hz / 1000)
            } else {
                format!("{hz}")
            };
            (index, text)
        })
        .collect()
}

/// Decibel gridlines for the top, middle, and bottom of the plot.
fn db_labels(min: i16, max: i16, fraction_digits: i16) -> Vec<(i16, String)> {
    [max, (min + max) / 2, min]
        .into_iter()
        .map(|raw| (raw, format_db(raw, fraction_digits)))
        .collect()
}

fn format_db(raw: i16, fraction_digits: i16) -> String {
    let divisor = 10f32.powi(i32::from(fraction_digits.max(0)));
    let db = (raw as f32 / divisor * 10.0).round() / 10.0;
    if db == db.trunc() {
        format!("{db:+.0}")
    } else {
        format!("{db:+.1}")
    }
}

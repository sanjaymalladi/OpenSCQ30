//! Reusable redesign components: page frame, headers, cards, mode choices,
//! status rows, and preset cards.
//!
//! All surfaces come from the active COSMIC theme (`theme::Container::Card`,
//! `button::suggested` for the single primary action); no hard-coded dark
//! colors, so the existing light theme keeps working. Copy must be localized
//! by callers with `fl!` before being passed in; components never bake in UI
//! text. Hierarchy uses size and weight, never color alone.

use std::borrow::Cow;

use cosmic::{
    Apply, Element,
    iced::{
        Background, Color, Length, Shadow, Vector,
        core::{Border, font::Font},
    },
    theme, widget,
};

/// Spacing scale (px) shared by redesigned pages: 8 / 12 / 16.
/// Larger steps are composed from these (e.g. `SPACE_M + SPACE_S`).
pub const SPACE_XS: u16 = 8;
/// See [`SPACE_XS`].
pub const SPACE_S: u16 = 12;
/// See [`SPACE_XS`].
pub const SPACE_M: u16 = 16;
/// Maximum content width so controls do not stretch on wide windows.
pub const PAGE_MAX_WIDTH: f32 = 960.0;
/// Title text size for page headers.
pub const TITLE_SIZE: u16 = 24;
/// Reference-design accent red, used for active/selected states only.
pub const ACCENT_RED: Color = Color::from_rgb8(0xFF, 0x30, 0x45);
/// Healthy-battery indicator green.
pub const HEALTHY_GREEN: Color = Color::from_rgb8(0x4C, 0xAF, 0x50);
/// Low-battery amber.
pub const BATTERY_AMBER: Color = Color::from_rgb8(0xE5, 0xA8, 0x00);
/// Muted secondary text from the reference tokens.
pub const MUTED_GREY: Color = Color::from_rgb8(0xA6, 0xA6, 0xA6);
/// Inactive glyph grey from the reference tokens.
pub const INACTIVE_ICON_GREY: Color = Color::from_rgb8(0xCF, 0xCF, 0xCF);
/// Diameter of the circular ambient-sound mode buttons.
pub const MODE_CIRCLE_DIAMETER: f32 = 76.0;

/// Serif display font for hero titles. Falls back to the system default serif
/// (or sans) where Georgia is unavailable.
pub fn display_font() -> Font {
    Font::with_name("Georgia")
}

/// Large serif hero title, e.g. the selected product name.
pub fn hero_title<'a, M>(title: impl Into<Cow<'a, str>> + 'a) -> Element<'a, M>
where
    M: Clone + 'static,
{
    widget::text(title).size(54).font(display_font()).into()
}

/// Scrollable page frame with centered maximum content width.
pub fn page<M>(content: Element<'_, M>) -> Element<'_, M>
where
    M: Clone + 'static,
{
    widget::scrollable(
        widget::container(content)
            .width(Length::Fill)
            .max_width(PAGE_MAX_WIDTH)
            .center_x(Length::Fill),
    )
    .into()
}

/// Page header: large title plus an optional supporting description.
pub fn page_header<'a, M>(
    title: impl Into<Cow<'a, str>> + 'a,
    description: Option<impl Into<Cow<'a, str>> + 'a>,
) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let mut column = widget::column![widget::text(title).size(TITLE_SIZE)].spacing(SPACE_XS);
    if let Some(description) = description {
        column = column.push(widget::text(description).size(14));
    }
    column.into()
}

/// Feature card: theme `Card` surface grouping one setting with its summary.
pub fn feature_card<'a, M>(
    title: impl Into<Cow<'a, str>> + 'a,
    description: Option<impl Into<Cow<'a, str>> + 'a>,
    content: Element<'a, M>,
) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let serif = cosmic::iced::Font {
        family: cosmic::iced::font::Family::Serif,
        ..Default::default()
    };
    let mut column = widget::column![widget::text(title).font(serif).size(22)].spacing(SPACE_XS);
    if let Some(description) = description {
        column = column.push(widget::text(description).size(14));
    }
    widget::container(column.push(content).spacing(SPACE_S))
        .class(theme::Container::Card)
        .padding(SPACE_M)
        .width(Length::Fill)
        .into()
}

/// Mutually-exclusive mode choices (e.g. Normal / Transparency / Noise
/// Cancellation). Each option is `(label, is_active, message)`. The active
/// option uses the theme accent style; inactive options stay neutral. The full
/// option area is clickable, not just an icon.
pub fn mode_choices<'a, M>(options: Vec<(String, bool, M)>) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let mut row = widget::row![].spacing(SPACE_S);
    for (label, active, message) in options {
        let button = if active {
            widget::button::suggested(label)
        } else {
            widget::button::standard(label)
        };
        row = row.push(button.width(Length::Fill).on_press(message));
    }
    row.into()
}

/// Circular ambient-sound mode selector matching the reference dashboard:
/// red glowing circle when active, charcoal otherwise, thin white line glyphs
/// with the label beneath. Each option is `(label, glyph_key, is_active,
/// message)`; unknown glyph keys fall back to the noise-canceling glyph.
/// Labels always render, so a missing glyph degrades gracefully.
pub fn mode_icon_choices<'a, M>(options: Vec<(String, &'static str, bool, M)>) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let mut row = widget::row![].spacing(SPACE_M);
    for (label, icon, active, message) in options {
        let glyph = match icon {
            "normal" => AmbientGlyph::Normal,
            "transparency" => AmbientGlyph::Transparency,
            _ => AmbientGlyph::NoiseCanceling,
        };
        let circle = widget::container(
            widget::canvas(AmbientModeIcon::new(
                glyph,
                if active {
                    Color::WHITE
                } else {
                    INACTIVE_ICON_GREY
                },
            ))
            .width(Length::Fixed(44.0))
            .height(Length::Fixed(44.0)),
        )
        .width(Length::Fixed(MODE_CIRCLE_DIAMETER))
        .height(Length::Fixed(MODE_CIRCLE_DIAMETER))
        .align_x(cosmic::iced::Alignment::Center)
        .align_y(cosmic::iced::Alignment::Center)
        .class(if active {
            theme::Container::custom(active_mode_circle)
        } else {
            theme::Container::custom(inactive_mode_circle)
        });
        row = row.push(
            widget::column![
                widget::mouse_area(circle).on_press(message),
                widget::text(label).size(14),
            ]
            .spacing(SPACE_XS)
            .align_x(cosmic::iced::Alignment::Center)
            .width(Length::Fill),
        );
    }
    row.into()
}

fn active_mode_circle(_theme: &cosmic::Theme) -> widget::container::Style {
    widget::container::Style {
        icon_color: Some(Color::WHITE),
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(ACCENT_RED)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 38.0.into(),
        },
        shadow: Shadow {
            color: ACCENT_RED,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 18.0,
        },
        snap: true,
    }
}

fn inactive_mode_circle(_theme: &cosmic::Theme) -> widget::container::Style {
    widget::container::Style {
        icon_color: Some(Color::WHITE),
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(Color::from_rgb8(0x26, 0x26, 0x26))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 38.0.into(),
        },
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
        snap: true,
    }
}

#[derive(Clone, Copy)]
enum AmbientGlyph {
    Normal,
    Transparency,
    NoiseCanceling,
}

struct AmbientModeIcon {
    glyph: AmbientGlyph,
    color: Color,
    cache: cosmic::widget::canvas::Cache,
}

impl AmbientModeIcon {
    fn new(glyph: AmbientGlyph, color: Color) -> Self {
        Self {
            glyph,
            color,
            cache: cosmic::widget::canvas::Cache::default(),
        }
    }
}

impl<M> cosmic::widget::canvas::Program<M, cosmic::Theme, cosmic::Renderer> for AmbientModeIcon
where
    M: Clone + 'static,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        _theme: &cosmic::Theme,
        bounds: cosmic::iced::Rectangle,
        _cursor: cosmic::iced::core::mouse::Cursor,
    ) -> Vec<cosmic::widget::canvas::Geometry<cosmic::iced::Renderer>> {
        use cosmic::iced::{Point, widget::canvas};

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Thin line glyphs: white when active, light grey otherwise.
            let color = self.color;
            let stroke = canvas::Stroke {
                style: canvas::stroke::Style::Solid(color),
                width: 2.0,
                line_cap: canvas::LineCap::Round,
                line_join: canvas::LineJoin::Round,
                line_dash: canvas::LineDash::default(),
            };
            match self.glyph {
                // Headphones: headband arc plus two ear cups.
                AmbientGlyph::Normal => {
                    let band = canvas::Path::new(|path| {
                        path.move_to(Point::new(10.0, 30.0));
                        path.quadratic_curve_to(Point::new(10.0, 10.0), Point::new(22.0, 10.0));
                        path.quadratic_curve_to(Point::new(34.0, 10.0), Point::new(34.0, 30.0));
                    });
                    frame.stroke(&band, stroke);
                    for left in [5.0, 31.0] {
                        let cup = canvas::Path::new(|path| {
                            path.rectangle(
                                Point::new(left, 26.0),
                                cosmic::iced::Size::new(8.0, 13.0),
                            );
                        });
                        frame.stroke(&cup, stroke);
                    }
                }
                // Ear: outer curve plus inner ridge.
                AmbientGlyph::Transparency => {
                    let outer = canvas::Path::new(|path| {
                        path.move_to(Point::new(29.0, 7.0));
                        path.quadratic_curve_to(Point::new(15.0, 9.0), Point::new(15.0, 22.0));
                        path.quadratic_curve_to(Point::new(15.0, 33.0), Point::new(27.0, 37.0));
                    });
                    frame.stroke(&outer, stroke);
                    let inner = canvas::Path::new(|path| {
                        path.move_to(Point::new(24.0, 15.0));
                        path.quadratic_curve_to(Point::new(19.0, 17.0), Point::new(19.0, 23.0));
                        path.quadratic_curve_to(Point::new(19.0, 29.0), Point::new(23.0, 31.0));
                    });
                    frame.stroke(&inner, stroke);
                }
                // Audio waveform with a minus overlay for cancellation.
                AmbientGlyph::NoiseCanceling => {
                    for (x, height) in [
                        (10.0, 8.0),
                        (16.0, 14.0),
                        (22.0, 22.0),
                        (28.0, 14.0),
                        (34.0, 8.0),
                    ] {
                        let top = 16.0 - height / 2.0;
                        frame.stroke(
                            &canvas::Path::line(Point::new(x, top), Point::new(x, top + height)),
                            stroke,
                        );
                    }
                    frame.stroke(
                        &canvas::Path::line(Point::new(23.0, 36.0), Point::new(35.0, 36.0)),
                        stroke,
                    );
                }
            }
        });
        vec![geometry]
    }
}

/// Title-left/status-right row for overview cards. `value` is the live
/// current state; `navigate` opens the full setting when present (chevron).
pub fn status_row<'a, M>(
    title: impl Into<Cow<'a, str>> + 'a,
    value: Option<impl Into<Cow<'a, str>> + 'a>,
    navigate: Option<M>,
) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let mut controls = widget::row![].spacing(SPACE_S);
    if let Some(value) = value {
        controls = controls.push(widget::text(value));
    }
    if let Some(message) = navigate {
        controls = controls.push(
            widget::icon::from_name("go-next-symbolic")
                .size(16)
                .apply(widget::button::icon)
                .on_press(message),
        );
    }
    widget::settings::item::builder(title)
        .control(controls)
        .into()
}

/// Preset card: name, concise summary, optional curve preview, and distinct
/// apply/edit actions. The current preset passes `apply` as `None` and shows
/// `applied_label` as non-interactive text instead of a button, so applying
/// and editing can never be confused. Accent styling is reserved for the
/// single apply action via `button::suggested`.
pub fn preset_card<'a, M>(
    name: impl Into<Cow<'a, str>> + 'a,
    summary: impl Into<Cow<'a, str>> + 'a,
    curve: Option<Element<'a, M>>,
    applied_label: Option<String>,
    apply: Option<(String, M)>,
    edit: Option<(String, M)>,
    delete: Option<(String, M)>,
) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let mut column = widget::column![widget::text(name).size(16), widget::text(summary).size(14)]
        .spacing(SPACE_XS);
    if let Some(curve) = curve {
        column = column.push(curve);
    }
    let mut actions = widget::row![].spacing(SPACE_S);
    match (applied_label, apply) {
        (_, Some((label, message))) => {
            actions = actions.push(widget::button::suggested(label).on_press(message));
        }
        (Some(label), None) => {
            actions = actions.push(widget::text(label));
        }
        (None, None) => {}
    }
    if let Some((label, message)) = edit {
        actions = actions.push(widget::button::standard(label).on_press(message));
    }
    if let Some((label, message)) = delete {
        actions = actions.push(widget::button::destructive(label).on_press(message));
    }
    widget::container(column.push(actions).spacing(SPACE_S))
        .class(theme::Container::Card)
        .padding(SPACE_M)
        .width(Length::Fill)
        .into()
}

/// Thin vertical divider for multi-column cards, using the muted token.
pub fn vertical_divider<'a, M>() -> Element<'a, M>
where
    M: Clone + 'static,
{
    widget::container(widget::text(""))
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .class(theme::Container::custom(divider_style))
        .into()
}

fn divider_style(_theme: &cosmic::Theme) -> widget::container::Style {
    widget::container::Style {
        icon_color: Some(Color::TRANSPARENT),
        text_color: Some(Color::TRANSPARENT),
        background: Some(Background::Color(Color::from_rgb8(0x30, 0x30, 0x30))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
        snap: true,
    }
}

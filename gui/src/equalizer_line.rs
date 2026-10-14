use std::marker::PhantomData;

use cosmic::{
    Element,
    cosmic_theme::palette::WithAlpha,
    iced::{Length, Point},
    widget::{self, canvas},
};

pub struct EqualizerLine<Message> {
    _message: PhantomData<Message>,
    cache: canvas::Cache,
    min: i16,
    max: i16,
    values: Vec<i16>,
    red: bool,
    /// extra x labels as (band index, text)
    x_bands: Vec<(usize, String)>,
    /// extra y gridlines as (raw value, text)
    y_db: Vec<(i16, String)>,
}

/// Plot margins reserving room for axis labels.
const PLOT_LEFT: f32 = 36.0;
const PLOT_TOP: f32 = 8.0;
const PLOT_RIGHT: f32 = 8.0;
const PLOT_BOTTOM: f32 = 18.0;

/// Smooths band points with Catmull-Rom splines. Shared with the interactive
/// equalizer graph so both render the same curve.
pub fn smooth_path(points: &[Point]) -> canvas::Path {
    canvas::Path::new(|path| {
        if points.is_empty() {
            return;
        }
        path.move_to(points[0]);
        let last = points.len() - 1;
        for i in 0..last {
            let before = points[i.saturating_sub(1)];
            let current = points[i];
            let next = points[i + 1];
            let after = points[(i + 2).min(last)];
            let control_a = Point::new(
                current.x + (next.x - before.x) / 6.0,
                current.y + (next.y - before.y) / 6.0,
            );
            let control_b = Point::new(
                next.x - (after.x - current.x) / 6.0,
                next.y - (after.y - current.y) / 6.0,
            );
            path.bezier_curve_to(control_a, control_b, next);
        }
    })
}

impl<Message> EqualizerLine<Message> {
    pub fn new(min: i16, max: i16, values: Vec<i16>) -> Self {
        Self {
            _message: PhantomData,
            cache: Default::default(),
            min,
            max,
            values,
            red: false,
            x_bands: Vec::new(),
            y_db: Vec::new(),
        }
    }

    /// Draws the curve in reference-design red instead of the theme color.
    pub fn red(mut self) -> Self {
        self.red = true;
        self
    }

    /// Sparse axis labels: x as band indices, y as raw values so labels
    /// always align with the curve mapping.
    pub fn labels(mut self, x_bands: Vec<(usize, String)>, y_db: Vec<(i16, String)>) -> Self {
        self.x_bands = x_bands;
        self.y_db = y_db;
        self
    }

    pub fn view(&self) -> Element<'_, Message> {
        widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn points(&self, width: f32, height: f32) -> Vec<Point> {
        let plot_width = (width - PLOT_LEFT - PLOT_RIGHT).max(1.0);
        let plot_height = (height - PLOT_TOP - PLOT_BOTTOM).max(1.0);
        let range = (self.max - self.min).max(1) as f32;
        let count = self.values.len();
        self.values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let fraction = if count > 1 {
                    i as f32 / (count - 1) as f32
                } else {
                    0.0
                };
                Point::new(
                    PLOT_LEFT + fraction * plot_width,
                    PLOT_TOP + (1.0 - (*v - self.min) as f32 / range) * plot_height,
                )
            })
            .collect()
    }

    fn point_y(&self, raw: i16, height: f32) -> f32 {
        let plot_height = (height - PLOT_TOP - PLOT_BOTTOM).max(1.0);
        let range = (self.max - self.min).max(1) as f32;
        PLOT_TOP + (1.0 - (raw - self.min) as f32 / range) * plot_height
    }

    fn point_x(&self, index: usize, width: f32) -> f32 {
        let plot_width = (width - PLOT_LEFT - PLOT_RIGHT).max(1.0);
        let count = self.values.len().max(2);
        PLOT_LEFT + (index.min(count - 1) as f32 / (count - 1) as f32) * plot_width
    }
}

impl<Message> canvas::Program<Message, cosmic::Theme, cosmic::Renderer> for EqualizerLine<Message> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        theme: &cosmic::Theme,
        bounds: cosmic::iced::Rectangle,
        _cursor: cosmic::iced::core::mouse::Cursor,
    ) -> Vec<canvas::Geometry<cosmic::iced::Renderer>> {
        let equalizer_line = self.cache.draw(renderer, bounds.size(), |frame| {
            let theme = theme.cosmic();
            let muted: cosmic::iced::Color = theme.on_bg_color().with_alpha(0.55).into();
            let faint: cosmic::iced::Color = theme.on_bg_color().with_alpha(0.22).into();
            let curve = if self.red {
                crate::components::ACCENT_RED
            } else {
                theme.on_bg_color().into()
            };
            let width = frame.width();
            let height = frame.height();

            let grid = canvas::Stroke {
                style: canvas::stroke::Style::Solid(faint),
                width: 1.0,
                line_cap: canvas::LineCap::Square,
                line_join: canvas::LineJoin::Bevel,
                line_dash: canvas::LineDash::default(),
            };
            if self.y_db.is_empty() {
                for y in [2.0, height / 2.0, height - 2.0] {
                    frame.stroke(
                        &canvas::Path::line(Point::new(0.0, y), Point::new(width, y)),
                        grid,
                    );
                }
            } else {
                for (raw, _) in &self.y_db {
                    let y = self.point_y(*raw, height);
                    frame.stroke(
                        &canvas::Path::line(
                            Point::new(PLOT_LEFT, y),
                            Point::new(width - PLOT_RIGHT, y),
                        ),
                        grid,
                    );
                }
            }
            for (index, _) in &self.x_bands {
                let x = self.point_x(*index, width);
                frame.stroke(
                    &canvas::Path::line(
                        Point::new(x, PLOT_TOP),
                        Point::new(x, height - PLOT_BOTTOM),
                    ),
                    grid,
                );
            }

            let points = self.points(width, height);
            frame.stroke(
                &smooth_path(&points),
                canvas::Stroke {
                    style: canvas::stroke::Style::Solid(curve),
                    width: 3f32,
                    line_cap: canvas::LineCap::Round,
                    line_join: canvas::LineJoin::Round,
                    line_dash: canvas::LineDash::default(),
                },
            );
            if self.red {
                for point in &points {
                    frame.fill(&canvas::Path::circle(*point, 3.0), curve);
                }
            }
            self.draw_labels(frame, muted);
        });
        vec![equalizer_line]
    }
}

impl<Message> EqualizerLine<Message> {
    fn draw_labels(
        &self,
        frame: &mut canvas::Frame<cosmic::iced::Renderer>,
        color: cosmic::iced::Color,
    ) {
        use cosmic::iced::core::text::{Alignment, LineHeight, Shaping};
        let height = frame.height();
        let width = frame.width();
        for (raw, content) in &self.y_db {
            frame.fill_text(canvas::Text {
                content: content.clone(),
                position: Point::new(4.0, self.point_y(*raw, height) - 6.0),
                max_width: 60.0,
                color,
                size: 10.0.into(),
                line_height: LineHeight::Relative(1.0),
                font: cosmic::iced::core::Font::default(),
                align_x: Alignment::Left,
                align_y: cosmic::iced::core::alignment::Vertical::Top,
                shaping: Shaping::Basic,
            });
        }
        let total = self.x_bands.len();
        for (i, (index, content)) in self.x_bands.iter().enumerate() {
            let x = self.point_x(*index, width);
            let (align_x, x) = if total > 1 && i == 0 {
                (Alignment::Left, PLOT_LEFT)
            } else if total > 1 && i + 1 == total {
                (Alignment::Right, width - PLOT_RIGHT)
            } else {
                (Alignment::Center, x)
            };
            frame.fill_text(canvas::Text {
                content: content.clone(),
                position: Point::new(x, height - 12.0),
                max_width: 60.0,
                color,
                size: 10.0.into(),
                line_height: LineHeight::Relative(1.0),
                font: cosmic::iced::core::Font::default(),
                align_x,
                align_y: cosmic::iced::core::alignment::Vertical::Top,
                shaping: Shaping::Basic,
            });
        }
    }
}

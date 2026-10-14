//! Interactive equalizer graph: draggable band handles on a live curve.
//!
//! Band count, bounds, and values come from the equalizer setting metadata;
//! nothing is hard-coded. Precise keyboard-accessible editing stays available
//! through the per-band spin buttons rendered alongside the graph, so the
//! canvas itself only needs pointer interaction.

use std::marker::PhantomData;

use cosmic::iced::core::mouse::{self, Cursor};
use cosmic::{
    cosmic_theme::palette::WithAlpha,
    iced::event::Event,
    iced::{Point, Rectangle},
    widget::canvas,
};

/// Drag state shared between `update` and `draw`.
#[derive(Default)]
pub struct DragState {
    dragged_band: Option<usize>,
}

pub struct EqualizerGraph<M> {
    cache: canvas::Cache,
    min: i16,
    max: i16,
    values: Vec<i16>,
    on_change: Box<dyn Fn(u8, i16) -> M>,
    _message: PhantomData<M>,
}

impl<M> EqualizerGraph<M> {
    pub fn new(
        min: i16,
        max: i16,
        values: Vec<i16>,
        on_change: impl Fn(u8, i16) -> M + 'static,
    ) -> Self {
        Self {
            cache: canvas::Cache::default(),
            min,
            max,
            values,
            on_change: Box::new(on_change),
            _message: PhantomData,
        }
    }

    fn point(&self, index: usize, width: f32, height: f32) -> Point<f32> {
        let padding = 12.0;
        let count = self.values.len().max(2);
        let normalized_x = index as f32 / (count - 1) as f32;
        let x = normalized_x * (width - padding * 2.0) + padding;
        let range = (self.max - self.min).max(1) as f32;
        let normalized_y = 1.0 - ((self.values[index] - self.min) as f32 / range);
        let y = normalized_y * (height - padding * 2.0) + padding;
        Point::new(x, y)
    }

    fn value_at(&self, y: f32, height: f32) -> i16 {
        let padding = 12.0;
        let usable = (height - padding * 2.0).max(1.0);
        let normalized = 1.0 - ((y - padding) / usable).clamp(0.0, 1.0);
        let range = (self.max - self.min).max(1) as f32;
        (self.min as f32 + normalized * range)
            .round()
            .clamp(self.min as f32, self.max as f32) as i16
    }

    fn band_at(&self, x: f32, width: f32, height: f32) -> Option<usize> {
        let padding = 12.0;
        let usable = (width - padding * 2.0).max(1.0);
        let count = self.values.len();
        if count == 0 {
            return None;
        }
        let position = ((x - padding) / usable).clamp(0.0, 1.0) * (count - 1) as f32;
        let index = position.round() as usize;
        let point = self.point(index, width, height);
        if (point.x - x).abs() <= grab_radius(width, count) {
            Some(index)
        } else {
            None
        }
    }
}

fn grab_radius(width: f32, count: usize) -> f32 {
    (width / count.max(1) as f32 / 2.0).clamp(10.0, 28.0)
}

impl<M> canvas::Program<M, cosmic::Theme, cosmic::Renderer> for EqualizerGraph<M>
where
    M: Clone + 'static,
{
    type State = DragState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<M>> {
        let position = cursor.position_over(bounds)?;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let band = self.band_at(position.x - bounds.x, bounds.width, bounds.height)?;
                state.dragged_band = Some(band);
                let value = self.value_at(position.y - bounds.y, bounds.height);
                Some(canvas::Action::publish((self.on_change)(band as u8, value)))
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let band = state.dragged_band?;
                let value = self.value_at(position.y - bounds.y, bounds.height);
                Some(canvas::Action::publish((self.on_change)(band as u8, value)))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragged_band = None;
                None
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        theme: &cosmic::Theme,
        bounds: cosmic::iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry<cosmic::iced::Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let cosmic = theme.cosmic();
            let accent: cosmic::iced::Color = cosmic.accent.base.into();
            for height in [2.0, frame.height() / 2.0, frame.height() - 2.0] {
                frame.stroke(
                    &canvas::Path::line(Point::new(0.0, height), Point::new(frame.width(), height)),
                    canvas::Stroke {
                        style: canvas::stroke::Style::Solid(
                            cosmic.on_bg_color().with_alpha(0.3).into(),
                        ),
                        width: 1.0,
                        line_cap: canvas::LineCap::Square,
                        line_join: canvas::LineJoin::Bevel,
                        line_dash: canvas::LineDash::default(),
                    },
                );
            }
            let width = frame.width();
            let height = frame.height();
            let points: Vec<Point> = (0..self.values.len())
                .map(|i| self.point(i, width, height))
                .collect();
            frame.stroke(
                &crate::equalizer_line::smooth_path(&points),
                canvas::Stroke {
                    style: canvas::stroke::Style::Solid(accent),
                    width: 3.0,
                    line_cap: canvas::LineCap::Round,
                    line_join: canvas::LineJoin::Round,
                    line_dash: canvas::LineDash::default(),
                },
            );
            for point in &points {
                frame.fill(&canvas::Path::circle(*point, 6.0), accent);
            }
        });
        vec![geometry]
    }
}

#[cfg(test)]
mod tests {
    use cosmic::{Element, iced::Length, widget};

    use super::*;

    #[test]
    fn value_mapping_clamps_to_bounds() {
        let graph = EqualizerGraph::new(-120, 134, vec![0; 8], |_, _| ());
        assert_eq!(graph.value_at(-100.0, 220.0), 134);
        assert_eq!(graph.value_at(320.0, 220.0), -120);
        let mid = graph.value_at(110.0, 220.0);
        assert!((-10..=10).contains(&mid), "middle maps near zero");
    }

    #[test]
    fn band_hit_testing_snaps_to_nearest() {
        let graph = EqualizerGraph::new(-120, 134, vec![0; 8], |_, _| ());
        // Near the edge handles snaps to the edge bands.
        assert_eq!(graph.band_at(0.0, 800.0, 220.0), Some(0));
        assert_eq!(graph.band_at(790.0, 800.0, 220.0), Some(7));
        // Far from any handle (midway gap is wider than the grab radius).
        assert_eq!(graph.band_at(289.0, 800.0, 220.0), None);
        // Empty values never hit.
        let empty = EqualizerGraph::new(-120, 134, Vec::new(), |_, _| ());
        assert_eq!(empty.band_at(400.0, 800.0, 220.0), None);
    }

    #[test]
    fn view_builds() {
        let graph = EqualizerGraph::new(-120, 134, vec![0; 8], |_, _| ());
        let _element: Element<'static, ()> = widget::canvas(graph)
            .width(Length::Fill)
            .height(Length::Fixed(220.0))
            .into();
    }
}

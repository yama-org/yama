use crate::widgets::theme::Theme;

use iced::mouse::Cursor;
use iced::widget::canvas::stroke::{self, Stroke};
use iced::widget::canvas::{self, Path};
use iced::{Point, Rectangle, Renderer, Vector};
use std::time::Instant;

#[derive(Debug)]
pub struct LoadingCircle {
    system_cache: canvas::Cache,
    start: Instant,
    now: Instant,
}

impl LoadingCircle {
    pub fn new() -> LoadingCircle {
        let now = Instant::now();
        LoadingCircle {
            system_cache: Default::default(),
            start: now,
            now,
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.system_cache.clear();
    }
}

impl<Message> canvas::Program<Message, Renderer<Theme>> for LoadingCircle {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer<Theme>,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        use std::f32::consts::PI;

        let system = self.system_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 5.0;
            let orbit = Path::circle(center, radius);

            frame.stroke(
                &orbit,
                Stroke {
                    style: stroke::Style::Solid(theme.text),
                    width: 2.0,
                    line_dash: canvas::LineDash {
                        offset: 0,
                        segments: &[3.0, 6.0],
                    },
                    ..Stroke::default()
                },
            );

            let elapsed = self.now - self.start;
            let rotation = (2.0 * PI / 60.0) * elapsed.as_secs() as f32
                + (2.0 * PI / 60_000.0) * elapsed.subsec_millis() as f32;

            frame.with_save(|frame| {
                frame.translate(Vector::new(center.x, center.y));
                frame.rotate(rotation * 50.0);
                frame.translate(Vector::new(radius, 0.0));

                let circle = Path::circle(Point::ORIGIN, radius / 6.0);
                frame.fill(&circle, theme.focus);
            });
        });

        vec![system]
    }
}

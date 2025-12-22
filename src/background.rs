use iced::mouse;
use iced::widget::canvas::{self, Geometry, Fill, Path, Stroke, Style};
use iced::{Color, Point, Rectangle, Renderer, Size};
use iced::border::Radius;

pub struct Background;

impl<Message> canvas::Program<Message> for Background {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geom = canvas::Cache::new().draw(renderer, bounds.size(), |frame| {
            let w = bounds.width as f32;
            let h = bounds.height as f32;

            // Base background (pure-ish black, slightly lifted so panels feel rich)
            frame.fill(
                &Path::rectangle(Point::ORIGIN, Size::new(w, h)),
                Fill {
                    style: Style::Solid(Color::from_rgb8(0x05, 0x05, 0x07)),
                    ..Fill::default()
                },
            );

            // Glow color (tweak to taste)
            let purple = Color::from_rgb8(0xB7, 0x63, 0xFF);

            // Large rounded rect matching window
            let r = 28.0;
            let outer = Path::rounded_rectangle(
                Point::new(10.0, 10.0),
                Size::new((w - 20.0).max(1.0), (h - 20.0).max(1.0)),
                Radius::from(r),
            );

            // Multi-stroke “bloom” (strongest near border)
            // Bigger width + lower alpha further out.
            for (i, (width, a)) in [
                (52.0, 0.06),
                (36.0, 0.08),
                (24.0, 0.12),
                (14.0, 0.18),
                (8.0,  0.25),
            ]
            .into_iter()
            .enumerate()
            {
                let _ = i; // keep if you want to vary radius too
                frame.stroke(
                    &outer,
                    Stroke {
                        width,
                        style: Style::Solid(Color { a, ..purple }),
                        ..Stroke::default()
                    },
                );
            }

            // Crisp inner edge highlight
            frame.stroke(
                &outer,
                Stroke {
                    width: 1.25,
                    style: Style::Solid(Color { a: 0.35, ..purple }),
                    ..Stroke::default()
                },
            );
        });

        vec![geom]
    }
}

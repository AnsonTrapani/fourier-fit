use iced::mouse;
use iced::widget::canvas::{self, Cache, Geometry};
use iced::Theme;
use iced::{Color, Point, Rectangle, Renderer, Size};
use crate::Message;

pub struct Background;

impl canvas::Program<Message> for Background {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let cache = Cache::new();
        let geom = cache.draw(renderer, bounds.size(), |frame| {
            let w = bounds.width as f32;
            let h = bounds.height as f32;

            // Base: pure black
            frame.fill_rectangle(
                Point::new(0.0, 0.0),
                Size::new(w, h),
                Color::BLACK,
            );

            // Fake a purple "hue" gradient near the top using translucent bands
            let bands = 40;
            let glow_h = (h * 0.35).max(1.0);
            for i in 0..bands {
                let t = i as f32 / (bands as f32);
                let y0 = t * glow_h;
                let band_h = glow_h / (bands as f32);

                // stronger at the very top, fades down
                let alpha = 0.22 * (1.0 - t).powf(1.8);

                frame.fill_rectangle(
                    Point::new(0.0, y0),
                    Size::new(w, band_h + 1.0),
                    Color { r: 0.55, g: 0.10, b: 0.85, a: alpha },
                );
            }
        });

        vec![geom]
    }
}

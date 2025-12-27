use iced::mouse;
use iced::widget::canvas::{self, Fill, Geometry, Path, Stroke, Style};
use iced::{Color, Point, Rectangle, Renderer, Size};
// use iced::border::Radius;

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
            let w = bounds.width;
            let h = bounds.height;

            // 1) Base: dark, slightly purple-tinted (so "starts darker")
            let base = Path::rectangle(Point::ORIGIN, Size::new(w, h));
            frame.fill(
                &base,
                Fill {
                    style: Style::Solid(Color::from_rgba8(0x07, 0x06, 0x0B, 1.0)),
                    ..Fill::default()
                },
            );

            // 2) Edge glow: many inset strokes that fade slowly toward center
            // Tune these 3 knobs:
            let steps: usize = 140; // more = smoother + "lasts longer"
            let glow_thickness = 0.38; // fraction of min(w,h) that glow extends inward
            let max_alpha = 0.38; // strength at the edge
            let falloff = 1.15; // lower (~0.9) = slower fade; higher = faster fade

            let s = w.min(h);
            let max_inset = s * glow_thickness;

            for i in 0..steps {
                let t = i as f32 / (steps - 1) as f32; // 0..1
                let inset = t * max_inset;

                // slow falloff toward center
                let a = (1.0 - t).powf(falloff) * max_alpha;

                // edge-purple (adjust these RGBs to taste)
                let glow = Color::from_rgba(0.21, 0.0, 0.31, a);

                // rounded rect hugging the window edge, then inset inward
                let x = inset;
                let y = inset;
                let rw = (w - 2.0 * inset).max(1.0);
                let rh = (h - 2.0 * inset).max(1.0);

                let rr = Path::rectangle(Point::new(x, y), Size::new(rw, rh));

                frame.stroke(
                    &rr,
                    Stroke {
                        // wider strokes near the edge feel more “glowy”
                        width: 2.4 + 2.6 * (1.0 - t),
                        style: Style::Solid(glow),
                        ..Stroke::default()
                    },
                );
            }
        });

        vec![geom]
    }
}

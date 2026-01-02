use iced::mouse;
use iced::widget::canvas::{self, Fill, Geometry, Path, Stroke, Style};
use iced::{Color, Point, Rectangle, Renderer, Size};

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

            // Base color definition
            let base = Path::rectangle(Point::ORIGIN, Size::new(w, h));
            frame.fill(
                &base,
                Fill {
                    style: Style::Solid(Color::from_rgba8(0x07, 0x06, 0x0B, 1.0)),
                    ..Fill::default()
                },
            );

            // Edge glow construction
            // Tunable params
            let steps: usize = 140; // Higher val makes smoother
            let glow_thickness = 0.38; // fraction that glow extends inward
            let max_alpha = 0.38; // strength of glow at edge
            let falloff = 1.15; // lower makes fade slower

            let s = w.min(h);
            let max_inset = s * glow_thickness;

            for i in 0..steps {
                let t = i as f32 / (steps - 1) as f32; // 0..1
                let inset = t * max_inset;

                // slow falloff toward center
                let a = (1.0 - t).powf(falloff) * max_alpha;

                // edge-purple
                let glow = Color::from_rgba(0.21, 0.0, 0.31, a);

                // rounded rect hugging the window edge
                let x = inset;
                let y = inset;
                let rw = (w - 2.0 * inset).max(1.0);
                let rh = (h - 2.0 * inset).max(1.0);

                let rr = Path::rectangle(Point::new(x, y), Size::new(rw, rh));

                frame.stroke(
                    &rr,
                    Stroke {
                        // wider strokes near the edge
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

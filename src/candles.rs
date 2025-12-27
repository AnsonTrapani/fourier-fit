use crate::Message;
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Fill, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

#[derive(Clone, Copy, Debug)]
pub struct Candle {
    pub t: f64, // time index (seconds, days, sample index... anything monotonic)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub struct CandlePanelView<'a> {
    pub zeros: Option<&'a [num_complex::Complex64]>,
    pub poles: Option<&'a [num_complex::Complex64]>,
    pub candles: Option<&'a [Candle]>,
    pub cache: &'a Cache,
    pub title: &'a str, // e.g. "Poles/Zeros + Time"
}

impl<'a> canvas::Program<Message> for CandlePanelView<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let geom = self
            .cache
            .draw(renderer, bounds.size(), |frame: &mut Frame| {
                let w = bounds.width;
                let h = bounds.height;

                // Panel
                let pad = 12.0_f32;
                let panel_x = pad;
                let panel_y = pad;
                let panel_w = (w - 2.0 * pad).max(1.0);
                let panel_h = (h - 2.0 * pad).max(1.0);

                let r = 22.0_f32;
                let panel = Path::rounded_rectangle(
                    Point::new(panel_x, panel_y),
                    Size::new(panel_w, panel_h),
                    iced::border::Radius::from(r),
                );

                frame.fill(
                    &panel,
                    Fill {
                        // use your helpers if you have them
                        style: iced::widget::canvas::Style::Solid(Color::from_rgb8(
                            0x0B, 0x0B, 0x0E,
                        )),
                        ..Fill::default()
                    },
                );

                frame.stroke(
                    &panel,
                    Stroke {
                        width: 1.0,
                        style: iced::widget::canvas::Style::Solid(Color {
                            a: 0.22,
                            ..Color::from_rgb8(0xA8, 0x3D, 0xFF)
                        }),
                        ..Stroke::default()
                    },
                );

                // Inner layout
                let inner_l = panel_x + 12.0;
                let inner_r = panel_x + panel_w - 12.0;
                let inner_t = panel_y + 10.0;
                let inner_b = panel_y + panel_h - 12.0;

                // Header region
                let header_h = 88.0_f32;
                let header_b = (inner_t + header_h).min(inner_b - 20.0);

                // Title
                frame.fill_text(Text {
                    content: self.title.into(),
                    position: Point::new(inner_l, inner_t),
                    color: Color::from_rgb8(0xD6, 0xD6, 0xD6),
                    size: 13.0.into(),
                    ..Text::default()
                });

                // Poles/Zeros text (2 columns)
                let text_y0 = inner_t + 18.0;
                let col_gap = 18.0;
                let col_w = ((inner_r - inner_l) - col_gap).max(1.0) * 0.5;
                let zeros_x = inner_l;
                let poles_x = inner_l + col_w + col_gap;

                let fmt_c = |z: num_complex::Complex64| -> String {
                    if z.im >= 0.0 {
                        format!("{:+.6} +{:.6}j", z.re, z.im)
                    } else {
                        format!("{:+.6} {:.6}j", z.re, z.im)
                    }
                };

                frame.fill_text(Text {
                    content: "Zeros (z-plane)".into(),
                    position: Point::new(zeros_x, text_y0),
                    color: Color::from_rgb8(0xB8, 0xB8, 0xB8),
                    size: 12.0.into(),
                    ..Text::default()
                });

                frame.fill_text(Text {
                    content: "Poles (z-plane)".into(),
                    position: Point::new(poles_x, text_y0),
                    color: Color::from_rgb8(0xB8, 0xB8, 0xB8),
                    size: 12.0.into(),
                    ..Text::default()
                });

                let mut y = text_y0 + 16.0;
                let line_h = 14.0_f32;

                let zeros = self.zeros.unwrap_or(&[]);
                let poles = self.poles.unwrap_or(&[]);
                let rows = zeros.len().max(poles.len()).min(4); // show first 4; tweak as you like

                for i in 0..rows {
                    if let Some(z) = zeros.get(i) {
                        frame.fill_text(Text {
                            content: fmt_c(*z),
                            position: Point::new(zeros_x, y),
                            color: Color::from_rgb8(0xD0, 0xD0, 0xD0),
                            size: 12.0.into(),
                            ..Text::default()
                        });
                    }
                    if let Some(p) = poles.get(i) {
                        frame.fill_text(Text {
                            content: fmt_c(*p),
                            position: Point::new(poles_x, y),
                            color: Color::from_rgb8(0xD0, 0xD0, 0xD0),
                            size: 12.0.into(),
                            ..Text::default()
                        });
                    }
                    y += line_h;
                }

                // Candle plot region
                let plot_l = inner_l;
                let plot_r = inner_r;
                let plot_t = header_b + 10.0;
                let plot_b = inner_b;
                let plot_w = (plot_r - plot_l).max(1.0);
                let plot_h = (plot_b - plot_t).max(1.0);

                // Plot border + light grid
                let grid = Stroke {
                    width: 1.0,
                    style: iced::widget::canvas::Style::Solid(Color::from_rgba8(
                        0xFF, 0xFF, 0xFF, 0.10,
                    )),
                    ..Stroke::default()
                };

                for k in 0..=4 {
                    let t = k as f32 / 4.0;
                    let yy = plot_t + t * plot_h;
                    frame.stroke(
                        &Path::line(Point::new(plot_l, yy), Point::new(plot_r, yy)),
                        grid,
                    );
                }

                frame.stroke(
                    &Path::rectangle(Point::new(plot_l, plot_t), Size::new(plot_w, plot_h)),
                    Stroke {
                        width: 1.0,
                        style: iced::widget::canvas::Style::Solid(Color::from_rgba8(
                            0xFF, 0xFF, 0xFF, 0.18,
                        )),
                        ..Stroke::default()
                    },
                );

                // Candles
                let candles = match self.candles {
                    Some(c) if c.len() >= 2 => c,
                    _ => {
                        let cx = panel_x + panel_w * 0.5;
                        let cy = plot_t + plot_h * 0.5;
                        frame.fill_text(Text {
                            content: "No time data".into(),
                            position: Point::new(cx, cy),
                            color: Color::from_rgb8(0xB8, 0xB8, 0xB8),
                            size: 14.0.into(),
                            align_x: iced::widget::text::Alignment::Center,
                            align_y: iced::alignment::Vertical::Center,
                            ..Text::default()
                        });
                        return;
                    }
                };

                // Range
                let mut tmin = f64::INFINITY;
                let mut tmax = f64::NEG_INFINITY;
                let mut vmin = f64::INFINITY;
                let mut vmax = f64::NEG_INFINITY;

                for c in candles {
                    if c.t.is_finite() && c.low.is_finite() && c.high.is_finite() {
                        tmin = tmin.min(c.t);
                        tmax = tmax.max(c.t);
                        vmin = vmin.min(c.low);
                        vmax = vmax.max(c.high);
                    }
                }
                if !(tmin.is_finite() && tmax.is_finite() && vmin.is_finite() && vmax.is_finite()) {
                    return;
                }
                if (tmax - tmin).abs() < 1e-12 {
                    tmax = tmin + 1.0;
                }
                if (vmax - vmin).abs() < 1e-12 {
                    vmax = vmin + 1.0;
                }

                // Pad y a bit
                let pady = 0.06 * (vmax - vmin);
                vmin -= pady;
                vmax += pady;

                let map_x = |t: f64| -> f32 {
                    let u = ((t - tmin) / (tmax - tmin)) as f32;
                    plot_l + u.clamp(0.0, 1.0) * plot_w
                };
                let map_y = |v: f64| -> f32 {
                    let u = ((v - vmin) / (vmax - vmin)) as f32;
                    plot_b - u.clamp(0.0, 1.0) * plot_h
                };

                // Candle width heuristic
                let n = candles.len().max(2) as f32;
                let cw = (plot_w / n).clamp(2.0, 10.0);
                let wick = Stroke {
                    width: 1.0,
                    style: iced::widget::canvas::Style::Solid(Color::from_rgba8(
                        0xFF, 0xFF, 0xFF, 0.55,
                    )),
                    ..Stroke::default()
                };

                for c in candles {
                    let x = map_x(c.t);

                    let y_high = map_y(c.high);
                    let y_low = map_y(c.low);
                    let y_open = map_y(c.open);
                    let y_close = map_y(c.close);

                    // wick
                    frame.stroke(
                        &Path::line(Point::new(x, y_high), Point::new(x, y_low)),
                        wick,
                    );

                    // body
                    let (top_y, bot_y) = if y_close < y_open {
                        (y_close, y_open)
                    } else {
                        (y_open, y_close)
                    };
                    let body_h = (bot_y - top_y).max(1.0);

                    let up = c.close >= c.open;
                    let body_color = if up {
                        Color::from_rgb8(0x33, 0xD1, 0x7A)
                    } else {
                        Color::from_rgb8(0xFF, 0x4D, 0x5A)
                    };

                    let body =
                        Path::rectangle(Point::new(x - cw * 0.5, top_y), Size::new(cw, body_h));

                    frame.fill(
                        &body,
                        Fill {
                            style: iced::widget::canvas::Style::Solid(body_color),
                            ..Fill::default()
                        },
                    );
                }
            });

        vec![geom]
    }
}

pub fn vec_to_candles(data: &[f64], num_per_candle: usize) -> Result<Vec<Candle>, String> {
    if num_per_candle == 0 {
        return Err("Cannot have a chunk size of zero in candle making function".into());
    }
    let mut candles: Vec<Candle> =
        Vec::with_capacity((data.len() as f64 / num_per_candle as f64).ceil() as usize);
    for (i, chunk) in data.chunks_exact(num_per_candle).enumerate() {
        candles.push(Candle {
            t: i as f64,
            open: chunk[0],
            close: chunk[chunk.len() - 1],
            high: chunk
                .iter()
                .fold(f64::NEG_INFINITY, |prev, curr| prev.max(*curr)),
            low: chunk
                .iter()
                .fold(f64::INFINITY, |prev, curr| prev.min(*curr)),
        })
    }
    Ok(candles)
}

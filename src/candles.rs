use crate::Message;
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Fill, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};
use std::default::Default;

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

                // Reserve space INSIDE the panel for right-side axis labels
                let y_axis_gutter = 64.0_f32; // tweak (56..80)
                let plot_r = inner_r - y_axis_gutter;

                let plot_t = header_b + 10.0;
                let plot_b = inner_b;

                let plot_w = (plot_r - plot_l).max(1.0);
                let plot_h = (plot_b - plot_t).max(1.0);

                // Axis label anchor (still inside the panel)
                let axis_x = plot_r + 8.0; // where tick labels start

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
                if (vmax - vmin).abs() < 1e-12 {
                    vmax = vmin + 1.0;
                }

                // Pad y a bit
                let pady = 0.06 * (vmax - vmin);
                vmin -= pady;
                vmax += pady;

                let grid = Stroke {
                    width: 1.0,
                    style: iced::widget::canvas::Style::Solid(Color::from_rgba8(
                        0xFF, 0xFF, 0xFF, 0.10,
                    )),
                    ..Stroke::default()
                };

                // Choose number of ticks like a chart
                let y_ticks = 9usize; // 7..11 feels good
                let tick_len = 6.0_f32;

                for k in 0..y_ticks {
                    let t = k as f32 / (y_ticks - 1) as f32; // 0..1 top->bottom
                    let yy = plot_t + t * plot_h;

                    // Horizontal grid line across plot
                    frame.stroke(
                        &Path::line(Point::new(plot_l, yy), Point::new(plot_r, yy)),
                        grid,
                    );

                    // Convert back to value for label (top is vmax, bottom is vmin)
                    let val = vmax - (t as f64) * (vmax - vmin);

                    // Small tick mark on the right edge
                    frame.stroke(
                        &Path::line(Point::new(plot_r, yy), Point::new(plot_r + tick_len, yy)),
                        Stroke {
                            width: 1.0,
                            style: iced::widget::canvas::Style::Solid(Color::from_rgba8(
                                0xFF, 0xFF, 0xFF, 0.35,
                            )),
                            ..Stroke::default()
                        },
                    );

                    // Tick label (in the gutter)
                    frame.fill_text(Text {
                        content: format!("{:.2}", val),
                        position: Point::new(axis_x + tick_len + 2.0, yy - 7.0),
                        color: Color::from_rgba8(0xFF, 0xFF, 0xFF, 0.65),
                        size: 11.0.into(),
                        ..Text::default()
                    });
                }

                // Plot border
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

                let map_y = |v: f64| -> f32 {
                    let u = ((v - vmin) / (vmax - vmin)) as f32;
                    plot_b - u.clamp(0.0, 1.0) * plot_h
                };

                // Candle width heuristic
                let n = candles.len().max(1) as f32;
                let slot_w = (plot_w / n).max(1.0);
                let candle_w = (slot_w * 0.70).clamp(2.0, 40.0);
                let gap = slot_w - candle_w;

                let x_for = |i: f32| -> f32 { plot_l + i * slot_w + gap * 0.5 };

                let wick_x_for = |i: f32| -> f32 { x_for(i) + candle_w * 0.5 };

                for c in candles {
                    // Skip bad data early (VERY important for wgpu stability)
                    if !(c.open.is_finite() && c.close.is_finite()) {
                        continue;
                    }

                    let x0 = x_for(c.t as f32);
                    let xc = wick_x_for(c.t as f32);

                    let y_open = map_y(c.open);
                    let y_close = map_y(c.close);
                    let y_high = map_y(c.high);
                    let y_low = map_y(c.low);

                    if !(y_open.is_finite()
                        && y_close.is_finite()
                        && y_high.is_finite()
                        && y_low.is_finite())
                    {
                        continue;
                    }

                    // Determine candle direction
                    let up = c.close >= c.open;

                    let color = if up {
                        Color::from_rgba8(0x2E, 0xE5, 0x9D, 0.90) // green
                    } else {
                        Color::from_rgba8(0xFF, 0x4D, 0x5A, 0.90) // red
                    };

                    // --------------------
                    // Wick
                    // --------------------
                    frame.stroke(
                        &Path::line(Point::new(xc, y_high), Point::new(xc, y_low)),
                        Stroke {
                            width: 1.0,
                            style: iced::widget::canvas::Style::Solid(color),
                            ..Stroke::default()
                        },
                    );

                    // --------------------
                    // Body
                    // --------------------
                    let y_top = y_open.min(y_close);
                    let y_bot = y_open.max(y_close);
                    let body_h = (y_bot - y_top).max(1.0);

                    let body = Path::rectangle(Point::new(x0, y_top), Size::new(candle_w, body_h));

                    frame.fill(
                        &body,
                        Fill {
                            style: iced::widget::canvas::Style::Solid(color),
                            ..Fill::default()
                        },
                    );

                    // Optional outline (nice on dark backgrounds)
                    frame.stroke(
                        &body,
                        Stroke {
                            width: 1.0,
                            style: iced::widget::canvas::Style::Solid(Color { a: 0.95, ..color }),
                            ..Stroke::default()
                        },
                    );
                }
                // ------------------------------------
                // Last-close dashed reference line
                // ------------------------------------
                if let Some(last) = candles
                    .iter()
                    .rev()
                    .find(|c| c.close.is_finite() && c.open.is_finite())
                {
                    let y_last = map_y(last.close);

                    if y_last.is_finite() {
                        let color = if last.close >= last.open {
                            Color::from_rgba8(0x2E, 0xE5, 0x9D, 0.90)
                        } else {
                            Color::from_rgba8(0xFF, 0x4D, 0x5A, 0.90)
                        };

                        // dashed line across plot (stops at plot_r)
                        frame.stroke(
                            &Path::line(Point::new(plot_l, y_last), Point::new(plot_r, y_last)),
                            Stroke {
                                width: 1.0,
                                style: iced::widget::canvas::Style::Solid(color),
                                line_dash: iced::widget::canvas::LineDash {
                                    segments: &[2.0, 4.0],
                                    offset: 0,
                                },
                                ..Stroke::default()
                            },
                        );

                        // label "pill" in the gutter; clamp y so it stays visible
                        let label = format!("{:.2}", last.close);
                        let font_px = 11.0_f32;

                        // crude text metrics (since iced 0.14 canvas renderer doesn't expose measure)
                        let approx_w = (label.chars().count() as f32) * font_px * 0.62;
                        let pad_x = 6.0_f32;
                        let pad_y = 3.0_f32;
                        let pill_w = approx_w + 2.0 * pad_x;
                        let pill_h = font_px + 2.0 * pad_y;

                        let mut pill_y = y_last - pill_h * 0.5;
                        pill_y = pill_y.clamp(plot_t, plot_b - pill_h);

                        let pill_x = (plot_r + 8.0).min(inner_r - pill_w - 2.0);

                        // background
                        frame.fill(
                            &Path::rounded_rectangle(
                                Point::new(pill_x, pill_y),
                                Size::new(pill_w, pill_h),
                                iced::border::Radius::from(6.0),
                            ),
                            Fill {
                                style: iced::widget::canvas::Style::Solid(color),
                                ..Fill::default()
                            },
                        );

                        // text
                        frame.fill_text(Text {
                            content: label,
                            position: Point::new(pill_x + pad_x, pill_y + pad_y - 1.0),
                            color: Color::from_rgba8(0x00, 0x00, 0x00, 0.92),
                            size: font_px.into(),
                            ..Text::default()
                        });
                    }
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandleLengths {
    #[default]
    Weekly,
    Monthly,
    Yearly,
}

impl From<CandleLengths> for usize {
    fn from(value: CandleLengths) -> Self {
        match value {
            CandleLengths::Weekly => 7,
            CandleLengths::Monthly => 30,
            CandleLengths::Yearly => 365,
        }
    }
}

impl std::fmt::Display for CandleLengths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CandleLengths::Weekly => "Weekly",
            CandleLengths::Monthly => "Monthly",
            CandleLengths::Yearly => "Yearly",
        };
        write!(f, "{s}")
    }
}

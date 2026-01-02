use crate::*;
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Fill, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

pub struct BodeView<'a> {
    pub freqs: Option<&'a [f64]>,
    /// Magnitude in dB for each frequency.
    pub mag_db: Option<&'a [f64]>,
    pub cache: &'a Cache,
    pub x_label: &'a str,
}

impl<'a> canvas::Program<Message> for BodeView<'a> {
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

                let pad = 12.0_f32;
                let panel_x = pad;
                let panel_y = pad;
                let panel_w = (w - 3.0 * pad).max(1.0);
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
                        style: iced::widget::canvas::Style::Solid(panel_bg()),
                        ..Fill::default()
                    },
                );

                frame.stroke(
                    &panel,
                    Stroke {
                        width: 1.0,
                        style: iced::widget::canvas::Style::Solid(panel_border()),
                        ..Stroke::default()
                    },
                );

                frame.stroke(
                    &panel,
                    Stroke {
                        width: 1.0,
                        style: iced::widget::canvas::Style::Solid(Color {
                            a: 0.22,
                            ..glow_purple()
                        }),
                        ..Stroke::default()
                    },
                );

                // Inner plotting rect
                let left = panel_x + 56.0; // extra space for dB labels
                let right = panel_x + panel_w - 12.0;
                let top = panel_y + 12.0;
                let bottom = panel_y + panel_h - 30.0;

                let plot_w = (right - left).max(1.0);
                let plot_h = (bottom - top).max(1.0);

                // Validate data
                let (freqs, mag_db) = match (self.freqs, self.mag_db) {
                    (Some(f), Some(m)) if f.len() == m.len() && f.len() >= 2 => (f, m),
                    _ => {
                        let size = 14.0;
                        let x_bias = 1.5 * size;
                        frame.fill_text(Text {
                            content: "No data loaded".into(),
                            position: Point::new(
                                ((left + right) * 0.5) - x_bias,
                                (top + bottom) * 0.5,
                            ),
                            color: label_color(),
                            size: size.into(),
                            align_x: iced::widget::text::Alignment::Center,
                            align_y: iced::alignment::Vertical::Center,
                            ..Text::default()
                        });
                        return;
                    }
                };

                // Find finite ranges; for log-x strictly positive frequencies
                let mut f_min = f64::INFINITY;
                let mut f_max = f64::NEG_INFINITY;
                let mut y_min = f64::INFINITY;
                let mut y_max = f64::NEG_INFINITY;

                for i in 0..freqs.len() {
                    let f = freqs[i];
                    let y = mag_db[i];
                    if f.is_finite() && y.is_finite() && f > 0.0 {
                        f_min = f_min.min(f);
                        f_max = f_max.max(f);
                        y_min = y_min.min(y);
                        y_max = y_max.max(y);
                    }
                }

                if !f_min.is_finite() || !f_max.is_finite() || f_min <= 0.0 || f_max <= 0.0 {
                    frame.fill_text(Text {
                        content: "Bode X requires positive frequencies".into(),
                        position: Point::new(left, top),
                        color: label_color(),
                        size: 14.0.into(),
                        ..Text::default()
                    });
                    return;
                }

                if !y_min.is_finite() || !y_max.is_finite() {
                    return;
                }

                if (y_max - y_min).abs() < 1e-12 {
                    let mid = 0.5 * (y_max + y_min);
                    y_min = mid - 1.0;
                    y_max = mid + 1.0;
                } else {
                    let pad_y = 0.08 * (y_max - y_min);
                    y_min -= pad_y;
                    y_max += pad_y;
                }

                let log_f_min = f_min.log10();
                let log_f_max = f_max.log10();
                let log_span = (log_f_max - log_f_min).max(1e-12);

                let map_x = |f: f64| -> f32 {
                    let t = ((f.log10() - log_f_min) / log_span) as f32;
                    left + t.clamp(0.0, 1.0) * plot_w
                };

                let map_y = |y: f64| -> f32 {
                    let t = ((y - y_min) / (y_max - y_min)) as f32;
                    bottom - t * plot_h
                };

                // Grid and box
                let grid = Stroke {
                    width: 1.0,
                    style: iced::widget::canvas::Style::Solid(grid_color()),
                    ..Stroke::default()
                };

                // Horizontal grid lines (5 total)
                for k in 0..=4 {
                    let t = k as f32 / 4.0;
                    let yy = top + t * plot_h;
                    frame.stroke(
                        &Path::line(Point::new(left, yy), Point::new(right, yy)),
                        grid,
                    );
                }

                // Vertical grid lines
                let decade_start = log_f_min.floor() as i32;
                let decade_end = log_f_max.ceil() as i32;
                for d in decade_start..=decade_end {
                    let f = 10f64.powi(d);
                    if f >= f_min && f <= f_max {
                        let xx = map_x(f);
                        frame.stroke(
                            &Path::line(Point::new(xx, top), Point::new(xx, bottom)),
                            grid,
                        );
                    }
                }

                frame.stroke(
                    &Path::rectangle(Point::new(left, top), Size::new(plot_w, plot_h)),
                    Stroke {
                        width: 1.0,
                        style: iced::widget::canvas::Style::Solid(grid_color()),
                        ..Stroke::default()
                    },
                );

                // Y tick labels (dB)
                let lbl = label_color();
                let y_mid = 0.5 * (y_min + y_max);
                for (val, yy) in [(y_max, top), (y_mid, (top + bottom) * 0.5), (y_min, bottom)] {
                    frame.fill_text(Text {
                        content: format!("{:.1} dB", val),
                        position: Point::new(panel_x + 6.0, yy - 7.0),
                        color: lbl,
                        size: 12.0.into(),
                        ..Text::default()
                    });
                }

                // X tick labels at decades
                let tick_stroke = Stroke {
                    width: 1.0,
                    style: iced::widget::canvas::Style::Solid(Color::from_rgb8(0x22, 0x22, 0x22)),
                    ..Stroke::default()
                };
                let tick_len = 6.0_f32;
                let x_label_y = bottom + 18.0;

                for d in decade_start..=decade_end {
                    let f = 10f64.powi(d);
                    if f < f_min || f > f_max {
                        continue;
                    }
                    let xx = map_x(f);
                    frame.stroke(
                        &Path::line(Point::new(xx, bottom), Point::new(xx, bottom + tick_len)),
                        tick_stroke,
                    );

                    // Value labels
                    frame.fill_text(Text {
                        content: format!("1e{}", d),
                        position: Point::new(xx - 14.0, x_label_y - 10.0),
                        color: lbl,
                        size: 12.0.into(),
                        ..Text::default()
                    });
                }

                frame.fill_text(Text {
                    content: self.x_label.into(),
                    position: Point::new(left + plot_w * 0.5 - 80.0, bottom + 22.0),
                    color: lbl,
                    size: 12.0.into(),
                    ..Text::default()
                });

                // Bode magnitude line
                let line_color = Color::from_rgb8(0x00, 0xB3, 0xFF);

                let mut started = false;
                let bode_path = Path::new(|p| {
                    for i in 0..freqs.len() {
                        let f = freqs[i];
                        let y = mag_db[i];
                        if !f.is_finite() || !y.is_finite() || f <= 0.0 {
                            continue;
                        }
                        let pt = Point::new(map_x(f), map_y(y));
                        if !started {
                            p.move_to(pt);
                            started = true;
                        } else {
                            p.line_to(pt);
                        }
                    }
                });

                frame.stroke(
                    &bode_path,
                    Stroke {
                        width: 2.0,
                        style: iced::widget::canvas::Style::Solid(line_color),
                        ..Stroke::default()
                    },
                );
            });

        vec![geom]
    }
}

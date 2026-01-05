use crate::*;
use iced::Theme;
use iced::border::Radius;
use iced::mouse;
use iced::widget::canvas::{self, Cache, Fill, Geometry, Path, Stroke, Style, Text};
use iced::{Color, Point, Rectangle, Renderer, Size};

pub struct SpectralView<'a> {
    pub fft_out: Option<&'a [f64]>,
    pub cache: &'a Cache,
}

impl<'a> canvas::Program<Message> for SpectralView<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
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
                Radius::from(r),
            );

            frame.fill(
                &panel,
                Fill {
                    style: Style::Solid(panel_bg()),
                    ..Fill::default()
                },
            );

            // Border
            frame.stroke(
                &panel,
                Stroke {
                    width: 1.0,
                    style: Style::Solid(panel_border()),
                    ..Stroke::default()
                },
            );

            frame.stroke(
                &panel,
                Stroke {
                    width: 1.0,
                    style: Style::Solid(Color {
                        a: 0.22,
                        ..glow_purple()
                    }),
                    ..Stroke::default()
                },
            );

            // Inner plotting rect
            let left = panel_x + 40.0;
            let right = panel_x + panel_w - 12.0;
            let top = panel_y + 12.0;
            let bottom = panel_y + panel_h - 28.0;

            let plot_w = (right - left).max(1.0);
            let plot_h = (bottom - top).max(1.0);

            // Decide how many points to draw
            if self.fft_out.is_none() {
                let size = 14.0;
                let x_bias = 0.9 * size;
                frame.fill_text(Text {
                    content: "No data loaded".into(),
                    position: Point::new(((left + right) * 0.5) - x_bias, (top + bottom) * 0.5),
                    color: label_color(),
                    size: size.into(),
                    align_x: iced::widget::text::Alignment::Center,
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                });
                return;
            }
            let fft_out = self.fft_out.unwrap();
            let n = fft_out.len();
            if n < 2 {
                // nothing meaningful to draw
                frame.fill_text(Text {
                    content: "Insufficient fft data".into(),
                    position: Point::new(left, top),
                    color: Color::from_rgb8(0x22, 0x22, 0x22),
                    size: 14.0.into(),
                    ..Text::default()
                });
                return;
            }

            // Y range from both series (raw + filtered if present)
            let ymin = 0f64;
            let mut ymax = f64::NEG_INFINITY;

            for &y in fft_out {
                if y.is_finite() {
                    ymax = ymax.max(y);
                }
            }

            if !ymin.is_finite() || !ymax.is_finite() {
                return;
            }

            // handle flat signal
            if (ymax - ymin).abs() < 1e-12 {
                let mid = 0.5 * (ymax + ymin);
                ymax = mid + 1.0;
            }

            let pad_y = 0.08 * (ymax - ymin);
            ymax += pad_y;

            let map_y = |y: f64| -> f32 {
                let t = ((y - ymin) / (ymax - ymin)) as f32;
                bottom - t * plot_h
            };

            // grid
            let grid = Stroke {
                width: 1.0,
                style: Style::Solid(grid_color()),
                ..Stroke::default()
            };

            for k in 0..=4 {
                let t = k as f32 / 4.0;
                let y = top + t * plot_h;
                frame.stroke(&Path::line(Point::new(left, y), Point::new(right, y)), grid);
            }
            for k in 0..=4 {
                let t = k as f32 / 4.0;
                let x = left + t * plot_w;
                frame.stroke(&Path::line(Point::new(x, top), Point::new(x, bottom)), grid);
            }

            // axes box
            frame.stroke(
                &Path::rectangle(Point::new(left, top), Size::new(plot_w, plot_h)),
                Stroke {
                    width: 1.0,
                    style: Style::Solid(grid_color()),
                    ..Stroke::default()
                },
            );

            // y ticks
            let label_color = label_color();
            let size = 12.0;

            let y_mid = 0.5 * (ymin + ymax);
            for (val, yy) in [(ymax, top), (y_mid, (top + bottom) * 0.5), (ymin, bottom)] {
                frame.fill_text(Text {
                    content: fmt_tick(val),
                    position: Point::new(panel_x + 6.0, yy - 6.0),
                    color: label_color,
                    size: size.into(),
                    ..Text::default()
                });
            }

            // --- bars ---
            let baseline_val = if ymin <= 0.0 && 0.0 <= ymax {
                0.0
            } else {
                ymin
            };
            let baseline_y = map_y(baseline_val);

            // Bar sizing
            let dx = plot_w / (n as f32);
            let gap = (dx * 0.15).min(3.0); // spacing between bars
            let bar_w = (dx - gap).max(1.0);

            let bar_color = Color::from_rgb8(0x00, 0x66, 0xCC);
            let mut max_bar_height = 0f64;

            for &num in fft_out {
                max_bar_height = f64::max(max_bar_height, num);
            }

            for (i, &y) in fft_out.iter().enumerate().skip(1) {
                if !y.is_finite() {
                    continue;
                }

                // x position centered in bin i
                let x = left + (i as f32) * dx + gap * 0.5;

                let y_px = map_y(y);

                // bar goes from baseline to y
                let (top_y, height) = if y_px < baseline_y {
                    (y_px, baseline_y - y_px) // positive relative to baseline
                } else {
                    (baseline_y, y_px - baseline_y) // negative relative to baseline
                };

                // Skip ultra-tiny bars
                if height <= max_bar_height as f32 * 0.01f32 {
                    continue;
                }

                let rect = Path::rectangle(Point::new(x, top_y), Size::new(bar_w, height.max(1.0)));
                frame.fill(
                    &rect,
                    Fill {
                        style: Style::Solid(bar_color),
                        ..Fill::default()
                    },
                );
            }

            let tick_stroke = Stroke {
                width: 1.0,
                style: Style::Solid(Color::from_rgb8(0x22, 0x22, 0x22)),
                ..Stroke::default()
            };

            let x_label_y = bottom + 16.0;
            let tick_len = 6.0_f32;

            // label 0 .. Nyquist (fs/2) in units cycles/day
            let nyq = 1. / 2.0;
            for k in 0..=4 {
                let t = k as f32 / 4.0;
                let x = left + t * plot_w;

                // tick mark
                frame.stroke(
                    &Path::line(Point::new(x, bottom), Point::new(x, bottom + tick_len)),
                    tick_stroke,
                );

                // value
                let f = (t as f64) * nyq;
                frame.fill_text(Text {
                    content: fmt_tick(f),
                    position: Point::new(x - 12.0, x_label_y - 10.),
                    color: label_color,
                    size: 12.0.into(),
                    ..Text::default()
                });
            }

            // x-axis unit label
            frame.fill_text(Text {
                content: "Frequency (cycles/day)".into(),
                position: Point::new(left + plot_w * 0.5 - 70.0, bottom + 20.0),
                color: label_color,
                size: 12.0.into(),
                ..Text::default()
            });
        });

        vec![geom]
    }
}

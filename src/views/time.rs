use iced::border::Radius;
use iced::mouse;
use iced::widget::canvas::{self, Cache, Fill, Geometry, Path, Stroke, Style, Text};
use iced::Theme;
use iced::{Color, Point, Rectangle, Renderer, Size};
use crate::*;

pub struct TimeSeriesPlotView<'a> {
    pub raw: Option<&'a [f64]>,
    pub filtered: Option<&'a [f64]>,
    pub cache: &'a Cache,
}

impl<'a> canvas::Program<Message> for TimeSeriesPlotView<'a> {
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

            // Border (optional but nice)
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

            let raw = match self.raw {
                Some(v) => v,
                None => {
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
            };

            // Decide how many points we can draw
            let n_raw = raw.len();
            // if n_raw < 2 {
            //     // nothing meaningful to draw
            //     frame.fill_text(Text {
            //         content: "No raw data".into(),
            //         position: Point::new((left + right) / 2., (top + bottom) / 2.),
            //         color: label_color(),
            //         size: 14.0.into(),
            //         ..Text::default()
            //     });
            //     return;
            // }

            let n = match self.filtered {
                Some(f) => n_raw.min(f.len()),
                None => n_raw,
            };
            // if n < 2 {
            //     return;
            // }

            // Y range from both series (raw + filtered if present)
            let mut ymin = f64::INFINITY;
            let mut ymax = f64::NEG_INFINITY;

            for &y in &raw[..n] {
                if y.is_finite() {
                    ymin = ymin.min(y);
                    ymax = ymax.max(y);
                }
            }
            if let Some(f) = self.filtered {
                for &y in &f[..n] {
                    if y.is_finite() {
                        ymin = ymin.min(y);
                        ymax = ymax.max(y);
                    }
                }
            }

            if !ymin.is_finite() || !ymax.is_finite() {
                return;
            }

            // handle flat signal
            if (ymax - ymin).abs() < 1e-12 {
                let mid = 0.5 * (ymax + ymin);
                ymin = mid - 1.0;
                ymax = mid + 1.0;
            }

            // add padding
            let pad_y = 0.08 * (ymax - ymin);
            ymin -= pad_y;
            ymax += pad_y;

            let map_x = |i: usize| -> f32 { left + (i as f32) * (plot_w / ((n - 1) as f32)) };
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

            // y ticks (min / mid / max)
            let label_color = label_color();
            let size = 12.0;

            let y_mid = 0.5 * (ymin + ymax);
            for (val, yy) in [(ymax, top), (y_mid, (top + bottom) * 0.5), (ymin, bottom)] {
                frame.fill_text(Text {
                    content: format!("{val:.1}"),
                    position: Point::new(panel_x + 6.0, yy - 6.0),
                    color: label_color,
                    size: size.into(),
                    ..Text::default()
                });
            }

            // draw raw line
            let raw_stroke = Stroke {
                width: 2.0,
                style: Style::Solid(Color::from_rgb8(0x00, 0x66, 0xCC)),
                ..Stroke::default()
            };

            let mut prev = None;
            for (i, &y) in raw.iter().enumerate().take(n) {
                if !y.is_finite() {
                    prev = None;
                    continue;
                }
                let p = Point::new(map_x(i), map_y(y));
                if let Some(q) = prev {
                    frame.stroke(&Path::line(q, p), raw_stroke);
                }
                prev = Some(p);
            }

            // draw filtered line (if available)
            if let Some(f) = self.filtered {
                let filt_stroke = Stroke {
                    width: 2.0,
                    style: Style::Solid(Color::from_rgb8(0xCC, 0x00, 0x00)),
                    ..Stroke::default()
                };

                let mut prev = None;
                for (i, &y) in f.iter().enumerate().take(n) {
                    if !y.is_finite() {
                        prev = None;
                        continue;
                    }
                    let p = Point::new(map_x(i), map_y(y));
                    if let Some(q) = prev {
                        frame.stroke(&Path::line(q, p), filt_stroke);
                    }
                    prev = Some(p);
                }
            }

            // legend
            frame.fill_text(Text {
                content: if self.filtered.is_some() {
                    "raw (blue) / filtered (red)".into()
                } else {
                    "raw (blue)".into()
                },
                position: Point::new(left, bottom + 8.0),
                color: label_color,
                size: 12.0.into(),
                ..Text::default()
            });
        });

        vec![geom]
    }
}

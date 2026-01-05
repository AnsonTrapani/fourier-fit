use crate::*;
use iced::Theme;
use iced::border::Radius;
use iced::mouse;
use iced::widget::canvas::{self, Cache, Fill, Geometry, Path, Stroke, Style, Text};
use iced::{Color, Point, Rectangle, Renderer, Size};
use num_complex::Complex;

pub struct PzPlotView<'a> {
    pub zeros: Option<&'a [Complex<f64>]>,
    pub poles: Option<&'a [Complex<f64>]>,
    pub cache: &'a Cache,
}

impl<'a> canvas::Program<Message> for PzPlotView<'a> {
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

            // Panel inset
            let pad = 12.0_f32;

            let panel_x = pad;
            let panel_y = pad;
            let panel_w = (w - 2.0 * pad).max(1.0);
            let panel_h = (h - 2.0 * pad).max(1.0);

            // "Squircle-ish" radius
            let r = 22.0_f32;

            let panel = Path::rounded_rectangle(
                Point::new(panel_x, panel_y),
                Size::new(panel_w, panel_h),
                Radius::from(r),
            );

            // background panel
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

            if self.zeros.is_none() && self.poles.is_none() {
                let size = 14.0;
                let x_bias = 1.3 * size;
                let left = panel_x + 56.0;
                let right = panel_x + panel_w - 12.0;
                let top = panel_y + 12.0;
                let bottom = panel_y + panel_h - 30.0;
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

            // Now draw inside the panel area
            let inner_w = panel_w;
            let inner_h = panel_h;
            let origin = Point::new(panel_x, panel_y);
            let center = Point::new(origin.x + inner_w * 0.5, origin.y + inner_h * 0.5);

            let s = inner_w.min(inner_h);
            let plot_r = s * 0.42;

            let grid_stroke = Stroke {
                width: 1.0,
                style: Style::Solid(grid_color()),
                ..Stroke::default()
            };

            for k in [-1.0_f32, -0.5, 0.0, 0.5, 1.0] {
                let x = center.x + k * plot_r;
                frame.stroke(
                    &Path::line(Point::new(x, origin.y), Point::new(x, origin.y + inner_h)),
                    grid_stroke,
                );
            }

            for k in [-1.0_f32, -0.5, 0.0, 0.5, 1.0] {
                let y = center.y - k * plot_r;
                frame.stroke(
                    &Path::line(Point::new(origin.x, y), Point::new(origin.x + inner_w, y)),
                    grid_stroke,
                );
            }

            let to_px = |z: Complex<f64>| -> Point {
                Point::new(
                    center.x + (z.re as f32) * plot_r,
                    center.y - (z.im as f32) * plot_r,
                )
            };

            let axis_stroke = Stroke {
                width: 1.5,
                style: Style::Solid(grid_color()),
                ..Stroke::default()
            };

            // Axes confined to panel bounds
            frame.stroke(
                &Path::line(
                    Point::new(origin.x, center.y),
                    Point::new(origin.x + inner_w, center.y),
                ),
                axis_stroke,
            );
            frame.stroke(
                &Path::line(
                    Point::new(center.x, origin.y),
                    Point::new(center.x, origin.y + inner_h),
                ),
                axis_stroke,
            );

            // Unit circle
            frame.stroke(
                &Path::circle(center, plot_r),
                Stroke {
                    width: 1.0,
                    style: Style::Solid(grid_color()),
                    ..Stroke::default()
                },
            );

            let label_color = label_color();
            let label_size = 14.0;

            frame.fill_text(Text {
                content: "0".into(),
                position: Point::new(center.x + 4.0, center.y),
                color: label_color,
                size: label_size.into(),
                ..Text::default()
            });

            frame.fill_text(Text {
                content: "1".into(),
                position: Point::new(center.x + plot_r + 4.0, center.y),
                color: label_color,
                size: label_size.into(),
                ..Text::default()
            });

            frame.fill_text(Text {
                content: "-1".into(),
                position: Point::new(center.x - plot_r + 4.0, center.y),
                color: label_color,
                size: label_size.into(),
                ..Text::default()
            });

            frame.fill_text(Text {
                content: " j".into(),
                position: Point::new(center.x + 4.0, center.y - plot_r),
                color: label_color,
                size: label_size.into(),
                ..Text::default()
            });

            frame.fill_text(Text {
                content: "-j".into(),
                position: Point::new(center.x + 4.0, center.y + plot_r),
                color: label_color,
                size: label_size.into(),
                ..Text::default()
            });

            // Zeros:
            if let Some(zs) = self.zeros {
                for &z in zs {
                    if z.re.is_finite() && z.im.is_finite() {
                        let p = to_px(z);
                        frame.stroke(
                            &Path::circle(p, 5.0),
                            Stroke {
                                width: 2.0,
                                style: Style::Solid(Color::from_rgb8(0x00, 0x66, 0xCC)),
                                ..Stroke::default()
                            },
                        );
                    }
                }
            }

            // Poles
            if let Some(ps) = self.poles {
                for &p0 in ps {
                    if p0.re.is_finite() && p0.im.is_finite() {
                        let p = to_px(p0);
                        let d = 5.0;
                        let pole_stroke = Stroke {
                            width: 2.0,
                            style: Style::Solid(Color::from_rgb8(0xCC, 0x00, 0x00)),
                            ..Stroke::default()
                        };

                        frame.stroke(
                            &Path::line(Point::new(p.x - d, p.y - d), Point::new(p.x + d, p.y + d)),
                            pole_stroke,
                        );
                        frame.stroke(
                            &Path::line(Point::new(p.x - d, p.y + d), Point::new(p.x + d, p.y - d)),
                            pole_stroke,
                        );
                    }
                }
            }
        });

        vec![geom]
    }
}

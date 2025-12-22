use fourier_fit::bode::BodeView;
use fourier_fit::*;
use fourier_fit::filters::cutoff_period_to_nyquist;
use fourier_fit::background::Background;
use iced::border::Radius;
use iced::mouse;
use iced::widget::Canvas;
use iced::widget::canvas::{self, Cache, Fill, Geometry, Path, Stroke, Style, Text};
use iced::{
    Alignment, Element, Length, Theme,
    widget::{button, column, pick_list, row, scrollable, text, text_input, stack},
};
use iced::{Color, Point, Rectangle, Renderer, Size};
use num_complex::Complex;

pub fn main() -> iced::Result {
    iced::application(Gui::default, Gui::update, Gui::view)
        .theme(Theme::Dark)
        .centered()
        .run()
}

#[derive(Default)]
struct Gui {
    app: App,

    // Store inputs as Strings (best practice for text_input)
    cutoff_s: String,
    order_s: String,
    ripple_s: String,
    attenuation_s: String,

    // Output
    error: Option<String>,
    zeros_out: String,
    poles_out: String,
    plot_cache: Cache,
    ts_cache: Cache,
    fft_cache: Cache,
    bode_cache: Cache,
}

impl Gui {
    fn default() -> Self {
        let mut app = App::new();
        // Optional: populate demo data so Calculate works immediately
        app.set_demo_data();

        Self {
            app,
            cutoff_s: "4.2".into(),
            order_s: "4".into(),
            ripple_s: "5".into(),
            attenuation_s: "40".into(),
            error: None,
            zeros_out: String::new(),
            poles_out: String::new(),
            plot_cache: Cache::new(),
            ts_cache: Cache::new(),
            fft_cache: Cache::new(),
            bode_cache: Cache::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FilterChanged(t) => {
                self.app.set_filter_type(t);
            }
            Message::CutoffChanged(s) => self.cutoff_s = s,
            Message::OrderChanged(s) => self.order_s = s,
            Message::RippleChanged(s) => self.ripple_s = s,
            Message::AttenuationChanged(s) => self.attenuation_s = s,

            Message::LoadDemo => {
                self.app.set_demo_data();
                self.error = None;
            }

            Message::ClearOutput => {
                self.error = None;
                self.zeros_out.clear();
                self.poles_out.clear();
                self.plot_cache.clear();
                self.ts_cache.clear();
                self.fft_cache.clear();
                self.bode_cache.clear();
            }

            Message::Calculate => {
                self.error = None;

                // Parse inputs
                let cutoff = match self.cutoff_s.trim().parse::<f64>() {
                    Ok(v) => match cutoff_period_to_nyquist(v) {
                        Ok(w) => w,
                        Err(e) => {
                            self.error = Some(e);
                            return;
                        }
                    },
                    Err(e) => {
                        self.error = Some(format!("cutoff parse error: {e}"));
                        return;
                    }
                };
                let order = match self.order_s.trim().parse::<usize>() {
                    Ok(v) => v,
                    Err(e) => {
                        self.error = Some(format!("order parse error: {e}"));
                        return;
                    }
                };
                let ripple = match self.ripple_s.trim().parse::<f64>() {
                    Ok(v) => v,
                    Err(e) => {
                        self.error = Some(format!("ripple parse error: {e}"));
                        return;
                    }
                };
                let attenuation = match self.attenuation_s.trim().parse::<f64>() {
                    Ok(v) => v,
                    Err(e) => {
                        self.error = Some(format!("attenuation parse error: {e}"));
                        return;
                    }
                };

                self.app.set_cutoff(cutoff);
                self.app.set_order(order);
                self.app.set_ripple(ripple);
                self.app.set_attenuation(attenuation);

                // Run your computation
                if let Err(e) = self.app.filter() {
                    self.error = Some(e);
                    return;
                }
                if let Err(e) = self.app.fft_filtered() {
                    self.error = Some(e);
                    return;
                }
                if let Err(e) = self.app.generate_bode() {
                    self.error = Some(e);
                    return;
                }

                // Format output (poles/zeros are Option<Vec<Complex<f64>>> in your App)
                self.zeros_out = match &self.app.zeros {
                    Some(z) if !z.is_empty() => z
                        .iter()
                        .map(|c| format!("{:+.6} {:+.6}j", c.re, c.im))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    _ => "(none)".into(),
                };

                self.poles_out = match &self.app.poles {
                    Some(p) if !p.is_empty() => p
                        .iter()
                        .map(|c| format!("{:+.6} {:+.6}j", c.re, c.im))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    _ => "(none)".into(),
                };
                self.plot_cache.clear();
                self.ts_cache.clear();
                self.fft_cache.clear();
                self.bode_cache.clear();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let filter_options = [
            FilterType::BUTTERWORTH,
            FilterType::CHEBYSHEV1,
            FilterType::CHEBYSHEV2,
        ];

        let controls = column![
            row![
                text("Filter:").width(Length::Shrink),
                pick_list(
                    filter_options,
                    Some(self.app.filter),
                    Message::FilterChanged
                )
                .width(Length::Fill),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text("Cutoff period (days):").width(Length::Shrink),
                text_input("e.g. 0.25", &self.cutoff_s)
                    .on_input(Message::CutoffChanged)
                    .width(180),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text("Order:").width(Length::Shrink),
                text_input("e.g. 4", &self.order_s)
                    .on_input(Message::OrderChanged)
                    .width(120),
                text("Ripple (dB):").width(Length::Shrink),
                text_input("e.g. 5", &self.ripple_s)
                    .on_input(Message::RippleChanged)
                    .width(120),
                text("Attenuation (dB):").width(Length::Shrink),
                text_input("e.g. 40", &self.attenuation_s)
                    .on_input(Message::AttenuationChanged)
                    .width(120),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                button("Generate demo data").on_press(Message::LoadDemo),
                button("Calculate").on_press(Message::Calculate),
                button("Clear").on_press(Message::ClearOutput),
            ]
            .spacing(12),
            if let Some(err) = &self.error {
                text(format!("Error: {err}"))
            } else {
                text("")
            }
        ]
        .spacing(14);

        let output = row![
            column![
                text("Zeros (z-plane)"),
                scrollable(text(&self.zeros_out)).height(220)
            ]
            .width(Length::FillPortion(1))
            .spacing(8),
            column![
                text("Poles (z-plane)"),
                scrollable(text(&self.poles_out)).height(220)
            ]
            .width(Length::FillPortion(1))
            .spacing(8),
        ]
        .spacing(16);

        let pz = Canvas::new(PzPlotView {
            zeros: self.app.zeros.as_deref(),
            poles: self.app.poles.as_deref(),
            cache: &self.plot_cache,
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let filter_tf_bode = Canvas::new(BodeView {
            freqs: if self.app.bode_plot.is_some() {Some(&self.app.bode_plot.as_ref().unwrap().0)} else {None},
            mag_db: if self.app.bode_plot.is_some() {Some(&self.app.bode_plot.as_ref().unwrap().1)} else {None},
            cache: &self.bode_cache,
            x_label: "Frequency (cycles/day)"
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let filtered = self
            .app
            .filtered_data
            .as_ref()
            .map(|f| f.filtered_data.as_slice());

        let ts = Canvas::new(TimeSeriesPlotView {
            raw: self.app.raw_data.as_slice(),
            filtered,
            cache: &self.ts_cache,
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let fft = Canvas::new(SpectralView {
            fft_out: self.app.data_spectrum.as_deref(),
            cache: &self.fft_cache,
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let content = row![
            column![controls, output].padding(16).spacing(16),
            column![row![pz, filter_tf_bode].padding(16).spacing(16), ts, fft].padding(16).spacing(16),
        ];

        stack![
            Canvas::new(Background).width(Length::Fill).height(Length::Fill),
            content,
        ]
        .into()
    }
}

struct PzPlotView<'a> {
    zeros: Option<&'a [Complex<f64>]>,
    poles: Option<&'a [Complex<f64>]>,
    cache: &'a Cache,
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
            let w = bounds.width as f32;
            let h = bounds.height as f32;

            // Panel inset (so we don't draw into the dark background region)
            let pad = 12.0_f32;

            let panel_x = pad;
            let panel_y = pad;
            let panel_w = (w - 2.0 * pad).max(1.0);
            let panel_h = (h - 2.0 * pad).max(1.0);

            // "Squircle-ish" radius: big rounded corners
            let r = 22.0_f32;

            let panel = Path::rounded_rectangle(
                Point::new(panel_x, panel_y),
                Size::new(panel_w, panel_h),
                Radius::from(r),
            );

            // White background panel
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
                    style: Style::Solid(Color { a: 0.22, ..glow_purple() }),
                    ..Stroke::default()
                },
            );

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

            // Zeros: small circles
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

            // Poles: X
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

struct TimeSeriesPlotView<'a> {
    raw: &'a [f64],
    filtered: Option<&'a [f64]>,
    cache: &'a Cache,
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
            let w = bounds.width as f32;
            let h = bounds.height as f32;

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
                    style: Style::Solid(Color { a: 0.22, ..glow_purple() }),
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

            // Decide how many points we can draw
            let n_raw = self.raw.len();
            if n_raw < 2 {
                // nothing meaningful to draw
                frame.fill_text(Text {
                    content: "No raw data".into(),
                    position: Point::new((left + right) / 2., (top + bottom) / 2.),
                    color: label_color(),
                    size: 14.0.into(),
                    ..Text::default()
                });
                return;
            }

            let n = match self.filtered {
                Some(f) => n_raw.min(f.len()),
                None => n_raw,
            };
            if n < 2 {
                return;
            }

            // Y range from both series (raw + filtered if present)
            let mut ymin = f64::INFINITY;
            let mut ymax = f64::NEG_INFINITY;

            for &y in &self.raw[..n] {
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
            for i in 0..n {
                let y = self.raw[i];
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
                for i in 0..n {
                    let y = f[i];
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

struct SpectralView<'a> {
    fft_out: Option<&'a [f64]>,
    cache: &'a Cache,
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
            let w = bounds.width as f32;
            let h = bounds.height as f32;

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
                    style: Style::Solid(Color { a: 0.22, ..glow_purple() }),
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

            // Decide how many points we can draw
            if self.fft_out.is_none() {
                frame.fill_text(Text {
                    content: "No fft data".into(),
                    position: Point::new((left + right) / 2., (top + bottom) / 2.),
                    color: label_color(),
                    size: 14.0.into(),
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

            // add padding
            let pad_y = 0.08 * (ymax - ymin);
            ymax += pad_y;

            // let map_x = |i: usize| -> f32 { left + (i as f32) * (plot_w / ((n - 1) as f32)) };
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
                    content: fourier_fit::fmt_tick(val),
                    position: Point::new(panel_x + 6.0, yy - 6.0),
                    color: label_color,
                    size: size.into(),
                    ..Text::default()
                });
            }

            // --- bars ---
            // Choose a baseline: 0 if it's within range, else ymin
            let baseline_val = if ymin <= 0.0 && 0.0 <= ymax { 0.0 } else { ymin };
            let baseline_y = map_y(baseline_val);

            // Bar sizing
            let dx = plot_w / (n as f32);
            let gap = (dx * 0.15).min(3.0);           // small spacing between bars
            let bar_w = (dx - gap).max(1.0);

            let bar_color = Color::from_rgb8(0x00, 0x66, 0xCC);
            let mut max_bar_height = 0f64;

            for &num in fft_out {
                max_bar_height = f64::max(max_bar_height, num);
            }

            for i in 1..n {
                let y = fft_out[i];
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

                // Skip ultra-tiny bars if you want:
                if height <= max_bar_height as f32 * 0.01f32 { continue; }

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

            // label 0 .. Nyquist (fs/2) in cycles/day
            let nyq = 1. / 2.0;
            for k in 0..=4 {
                let t = k as f32 / 4.0;
                let x = left + t * plot_w;

                // tick mark
                frame.stroke(&Path::line(Point::new(x, bottom), Point::new(x, bottom + tick_len)), tick_stroke);

                // value
                let f = (t as f64) * nyq;
                frame.fill_text(Text {
                    content: fourier_fit::fmt_tick(f),
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

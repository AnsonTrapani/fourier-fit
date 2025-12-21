use iced::{
    widget::{button, column, pick_list, row, scrollable, text, text_input},
    Alignment, Element, Length, Theme,
};
use iced::widget::Canvas;
use iced::widget::canvas::{self, Cache, Geometry, Path, Stroke, Fill, Text};
use iced::{Color, Point, Rectangle, Renderer, Size};
use iced::mouse;
use num_complex::Complex;
use fourier_fit::{App, FilterType, filters::cutoff_period_to_nyquist};

pub fn main() -> iced::Result {
    iced::application(Gui::default, Gui::update, Gui::view)
        .theme(Theme::Dark)
        .centered()
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    FilterChanged(FilterType),
    CutoffChanged(String),
    OrderChanged(String),
    RippleChanged(String),
    AttenuationChanged(String),
    LoadDemo,
    Calculate,
    ClearOutput,
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
            }

            Message::Calculate => {
                self.error = None;

                // Parse inputs
                let cutoff = match self.cutoff_s.trim().parse::<f64>() {
                    Ok(v) => match cutoff_period_to_nyquist(v) {
                        Ok(w) => w,
                        Err(e) => {self.error = Some(e); return;}
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
            column![text("Zeros (z-plane)"), scrollable(text(&self.zeros_out)).height(220)]
                .width(Length::FillPortion(1))
                .spacing(8),
            column![text("Poles (z-plane)"), scrollable(text(&self.poles_out)).height(220)]
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
            .height(300);

        column![controls, output, pz]
            .padding(16)
            .spacing(16)
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
            let s = w.min(h);
            let center = Point::new(w * 0.5, h * 0.5);

            let r = s * 0.42;
            let to_px = |z: Complex<f64>| -> Point {
                Point::new(center.x + (z.re as f32) * r, center.y - (z.im as f32) * r)
            };

            // axes
            frame.stroke(
                &Path::line(Point::new(0.0, center.y), Point::new(w, center.y)),
                Stroke { width: 1.0, ..Stroke::default() },
            );
            frame.stroke(
                &Path::line(Point::new(center.x, 0.0), Point::new(center.x, h)),
                Stroke { width: 1.0, ..Stroke::default() },
            );

            // unit circle
            frame.stroke(
                &Path::circle(center, r),
                Stroke { width: 1.0, ..Stroke::default() },
            );

            // zeros: draw small circles
            if let Some(zs) = self.zeros {
                for &z in zs {
                    if z.re.is_finite() && z.im.is_finite() {
                        let p = to_px(z);
                        frame.stroke(
                            &Path::circle(p, 5.0),
                            Stroke { width: 2.0, ..Stroke::default() },
                        );
                    }
                }
            }

            // poles: draw X
            if let Some(ps) = self.poles {
                for &p0 in ps {
                    if p0.re.is_finite() && p0.im.is_finite() {
                        let p = to_px(p0);
                        let d = 5.0;
                        frame.stroke(
                            &Path::line(Point::new(p.x - d, p.y - d), Point::new(p.x + d, p.y + d)),
                            Stroke { width: 2.0, ..Stroke::default() },
                        );
                        frame.stroke(
                            &Path::line(Point::new(p.x - d, p.y + d), Point::new(p.x + d, p.y - d)),
                            Stroke { width: 2.0, ..Stroke::default() },
                        );
                    }
                }
            }
        });

        vec![geom]
    }
}

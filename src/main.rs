use iced::{
    widget::{button, column, pick_list, row, scrollable, text, text_input},
    Alignment, Element, Length, Theme,
};

use fourier_fit::{App, FilterType}; // <-- replace with your crate name

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
}

impl Gui {
    fn default() -> Self {
        let mut app = App::new();
        // Optional: populate demo data so Calculate works immediately
        app.set_demo_data();

        Self {
            app,
            cutoff_s: "0.25".into(),
            order_s: "4".into(),
            ripple_s: "5".into(),
            attenuation_s: "40".into(),
            error: None,
            zeros_out: String::new(),
            poles_out: String::new(),
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
            }

            Message::Calculate => {
                self.error = None;

                // Parse inputs
                let cutoff = match self.cutoff_s.trim().parse::<f64>() {
                    Ok(v) => v,
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
            }
        }
    }

    fn view(&self) -> Element<Message> {
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
                text("Cutoff (normalized, 0..0.5):").width(Length::Shrink),
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

        column![controls, output]
            .padding(16)
            .spacing(16)
            .into()
    }
}
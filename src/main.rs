use fourier_fit::views;
use fourier_fit::*;
use fourier_fit::structures::data_modal;
use iced::widget::Canvas;
use iced::widget::canvas::Cache;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{button, column, pick_list, row, stack, text, text_input, container},
};

const BOLD: iced::Font = iced::Font::with_name("Inter ExtraBold");

pub fn main() -> iced::Result {
    iced::application(Gui::default, Gui::update, Gui::view)
        .theme(Theme::Dark)
        .centered()
        .run()
}

#[derive(Default)]
struct Gui {
    // Mathematics state
    app: App,

    // Data modal state
    modal_state: data_modal::DataModalState,

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
    candles_cache: Cache,
}

impl Gui {
    fn default() -> Self {
        let app = App::new();
        // Optional: populate demo data so Calculate works immediately
        // app.set_demo_data();

        Self {
            app,
            modal_state: data_modal::DataModalState::new(),
            cutoff_s: "".into(),
            order_s: "".into(),
            ripple_s: "".into(),
            attenuation_s: "".into(),
            error: None,
            zeros_out: String::new(),
            poles_out: String::new(),
            plot_cache: Cache::new(),
            ts_cache: Cache::new(),
            fft_cache: Cache::new(),
            bode_cache: Cache::new(),
            candles_cache: Cache::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FilterChanged(t) => {
                self.app.set_filter_type(t);
            }
            Message::CandleLengthsChanged(t) => {
                self.app.candle_length = t;
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
                self.candles_cache.clear();
            }

            Message::Calculate => {
                self.error = None;

                // Parse inputs
                let cutoff = match self.cutoff_s.trim().parse::<f64>() {
                    Ok(v) => match math::cutoff_period_to_nyquist(v) {
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
                self.candles_cache.clear();
            },
            Message::WeightSelectionChanged(s) => self.modal_state.weight_entry = s,
            Message::OpenDataModal => self.modal_state.show_modal = true,
            Message::CloseDataModal => self.modal_state.show_modal = false,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let filter_options = [
            structures::filters::FilterType::BUTTERWORTH,
            structures::filters::FilterType::CHEBYSHEV1,
            structures::filters::FilterType::CHEBYSHEV2,
        ];
        let candle_options = [
            structures::candle::CandleLengths::Weekly,
            structures::candle::CandleLengths::Monthly,
            structures::candle::CandleLengths::Yearly,
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
                text("Candle Lengths:").width(Length::Shrink),
                pick_list(
                    candle_options,
                    Some(self.app.candle_length),
                    Message::CandleLengthsChanged
                )
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text("Cutoff period (days):").width(Length::Shrink),
                text_input("e.g. 4.2", &self.cutoff_s)
                    .on_input(Message::CutoffChanged)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text("Order:").width(Length::Shrink),
                text_input("e.g. 4", &self.order_s)
                    .on_input(Message::OrderChanged)
                    .width(Length::FillPortion(1)),
                text("Ripple (dB):").width(Length::Shrink),
                text_input("e.g. 5", &self.ripple_s)
                    .on_input(Message::RippleChanged)
                    .width(Length::FillPortion(1)),
                text("Attenuation (dB):").width(Length::Shrink),
                text_input("e.g. 40", &self.attenuation_s)
                    .on_input(Message::AttenuationChanged)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                button("Edit/Load Data").on_press(Message::OpenDataModal),
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

        let pz = Canvas::new(views::pz::PzPlotView {
            zeros: self.app.zeros.as_deref(),
            poles: self.app.poles.as_deref(),
            cache: &self.plot_cache,
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let filter_tf_bode = Canvas::new(views::bode::BodeView {
            freqs: if self.app.bode_plot.is_some() {
                Some(&self.app.bode_plot.as_ref().unwrap().0)
            } else {
                None
            },
            mag_db: if self.app.bode_plot.is_some() {
                Some(&self.app.bode_plot.as_ref().unwrap().1)
            } else {
                None
            },
            cache: &self.bode_cache,
            x_label: "Frequency (cycles/day)",
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let filtered = self
            .app
            .filtered_data
            .as_ref()
            .map(|f| f.filtered_data.as_slice());

        let ts = Canvas::new(views::time::TimeSeriesPlotView {
            raw: self.app.raw_data.as_deref(),
            filtered,
            cache: &self.ts_cache,
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let fft = Canvas::new(views::frequency::SpectralView {
            fft_out: self.app.data_spectrum.as_deref(),
            cache: &self.fft_cache,
        })
        .width(Length::Fill)
        .height(Length::FillPortion(1));

        let candle_panel = Canvas::new(views::candles::CandlePanelView {
            zeros: self.app.zeros.as_deref(),
            poles: self.app.poles.as_deref(),
            candles: self.app.candles.as_deref(),
            cache: &self.candles_cache,
            title: "Candle View",
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let content = row![
            column![controls, text("Candle View").font(BOLD), candle_panel].padding(16).spacing(5),
            column![row![column![text("Pole/Zero Plot").font(BOLD), pz], column![text("Bode Plot").font(BOLD), filter_tf_bode]].spacing(5), text("Time Domain").font(BOLD), ts, text("Frequency Domain").font(BOLD), fft]
                .padding(16)
                .spacing(5),
        ];

        let main_stack = stack![
            Canvas::new(views::background::Background)
                .width(Length::Fill)
                .height(Length::Fill),
            content,
        ];
        if !self.modal_state.show_modal {
            return main_stack.into();
        }
        // --- Modal content (the “card”) ---
        let modal_card = container(
            column![
                text("Edit details").size(22),
                text_input("", &self.modal_state.weight_entry)
                    .on_input(Message::WeightSelectionChanged),
                row![
                    button("Close").on_press(Message::CloseDataModal),
                ]
                .spacing(12),
            ]
            .spacing(12)
            .padding(16),
        )
        .width(Length::Fixed(420.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb8(0x1f, 0x1f, 0x1f))),
            text_color: Some(iced::Color::WHITE),
            border: iced::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgb8(0x44, 0x44, 0x44),
            },
            ..Default::default()
        });

        // --- Scrim + centered card ---
        let overlay = container(
            container(modal_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: 0.0, g: 0.0, b: 0.0, a: 0.55, // translucent scrim
            })),
            ..Default::default()
        });

        // Stack base + overlay (overlay on top)
        stack![main_stack, overlay].into()
    }
}

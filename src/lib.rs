pub mod logic;
pub mod math;
pub mod structures;
pub mod views;
use directories::ProjectDirs;
use std::{io, path::PathBuf};

use iced::Color;
use math::{
    FilterData, NYQUIST_PERIOD, butterworth_filter, chebyshev_filter_1, chebyshev_filter_2,
};
use num_complex::Complex;

const DEFAULT_ORDER: usize = 4;
const DEFAULT_RIPPLE: f64 = 5.;
const DEFAULT_ATTENUATION: f64 = 40.;
pub const DEFAULT_FILENAME: &str = "fourier_fit_data.json";

#[derive(Default)]
pub struct App {
    pub raw_data: Option<Vec<f64>>,
    pub filter: structures::filters::FilterType,
    pub cutoff_freq: f64,
    pub filtered_data: Option<FilterData>,
    pub order: usize,
    pub ripple: f64,
    pub attenuation: f64,
    pub poles: Option<Vec<Complex<f64>>>,
    pub zeros: Option<Vec<Complex<f64>>>,
    pub bode_plot: Option<(Vec<f64>, Vec<f64>)>,
    pub data_spectrum: Option<Vec<f64>>,
    pub candles: Option<Vec<structures::candle::Candle>>,
    pub candle_length: structures::candle::CandleLengths,
}

impl App {
    pub fn new() -> Self {
        Self {
            raw_data: None,
            filter: structures::filters::FilterType::BUTTERWORTH,
            cutoff_freq: NYQUIST_PERIOD,
            filtered_data: None,
            order: DEFAULT_ORDER,
            ripple: DEFAULT_RIPPLE,
            attenuation: DEFAULT_ATTENUATION,
            poles: None,
            zeros: None,
            bode_plot: None,
            data_spectrum: None,
            candles: None,
            candle_length: structures::candle::CandleLengths::Weekly,
        }
    }

    pub fn filter(&mut self) -> Result<(), String> {
        let data = match self.raw_data.as_ref() {
            Some(v) => v,
            None => return Err(String::from("No data set")),
        };
        self.filtered_data = match self.filter {
            structures::filters::FilterType::BUTTERWORTH => {
                Some(butterworth_filter(data, self.cutoff_freq, self.order)?)
            }
            structures::filters::FilterType::CHEBYSHEV1 => Some(chebyshev_filter_1(
                data,
                self.cutoff_freq,
                self.order,
                self.ripple,
            )?),
            structures::filters::FilterType::CHEBYSHEV2 => Some(chebyshev_filter_2(
                data,
                self.cutoff_freq,
                self.order,
                self.attenuation,
            )?),
        };
        (self.zeros, self.poles) = match math::iir_zeros_poles_z(
            self.filtered_data.as_ref().unwrap().b.as_slice(),
            self.filtered_data.as_ref().unwrap().a.as_slice(),
        ) {
            Ok((z, p)) => (Some(z), Some(p)),
            Err(s) => return Err(s),
        };
        self.candles = structures::candle::vec_to_candles(
            self.raw_data.as_deref().unwrap(),
            self.candle_length.into(),
        )
        .ok();
        Ok(())
    }

    pub fn set_filter_type(&mut self, t: structures::filters::FilterType) {
        self.filter = t;
    }
    pub fn set_cutoff(&mut self, v: f64) {
        self.cutoff_freq = v;
    }
    pub fn set_order(&mut self, v: usize) {
        self.order = v;
    }
    pub fn set_ripple(&mut self, v: f64) {
        self.ripple = v;
    }
    pub fn set_attenuation(&mut self, v: f64) {
        self.attenuation = v;
    }

    pub fn set_app_data(&mut self, data: Vec<f64>) {
        self.raw_data = Some(data);
    }

    pub fn fft_filtered(&mut self) -> Result<(), String> {
        if let Some(data) = &self.filtered_data {
            self.data_spectrum = Some(math::rfft_mag(&data.filtered_data)?);
            Ok(())
        } else {
            Err(String::from("Filtering not complete"))
        }
    }

    pub fn generate_bode(&mut self) -> Result<(), String> {
        if let Some(data) = &self.filtered_data {
            self.bode_plot = Some(math::bode_mag_logspace(&data.b, &data.a, 1., 100));
            return Ok(());
        }
        Err(String::from("Filtering not complete"))
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    FilterChanged(structures::filters::FilterType),
    CutoffChanged(String),
    OrderChanged(String),
    RippleChanged(String),
    AttenuationChanged(String),
    LoadDemo,
    Calculate,
    ClearOutput,
    CandleLengthsChanged(structures::candle::CandleLengths),
    OpenDataModal,
    CloseDataModal,
    WeightSelectionChanged(String),
    NoOp,
    UpdateDate(iced_aw::date_picker::Date),
    SaveWeightSelection,
}

pub fn fmt_tick(v: f64) -> String {
    let av = v.abs();
    if (av > 0.0 && av < 0.01) || av >= 10_000.0 {
        format!("{v:.2e}")
    } else if av >= 100.0 {
        format!("{v:.0}")
    } else if av >= 10.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    }
}

pub fn panel_bg() -> Color {
    Color::from_rgb8(0x10, 0x10, 0x14)
} // dark panel
pub fn panel_border() -> Color {
    Color::from_rgb8(0x2A, 0x2A, 0x33)
} // subtle border
pub fn grid_color() -> Color {
    Color::from_rgb8(0xF8, 0xEF, 0xFF)
} // dark grid
pub fn label_color() -> Color {
    Color::from_rgb8(0xD6, 0xD6, 0xE2)
} // light text
pub fn glow_purple() -> Color {
    Color::from_rgb8(0xB7, 0x63, 0xFF)
} // accent

pub fn weight_file() -> Result<PathBuf, String> {
    let proj = ProjectDirs::from("", "", "fourier-fit")
        .ok_or("Could not determine config directory".to_string())?;
    Ok(proj.config_dir().join(DEFAULT_FILENAME))
}

pub fn create_file_perhaps(file_path: &std::path::PathBuf) -> io::Result<()> {
    let ok_res = std::fs::exists(file_path)?;
    if !ok_res {
        if let Some(parent_directory) = (file_path).parent() {
            std::fs::create_dir_all(parent_directory)?;
        }
        std::fs::File::create(file_path)?;
    }
    Ok(())
}

fn is_file_empty(file_path: &std::path::Path) -> bool {
    match std::fs::metadata(file_path) {
        Ok(metadata) => metadata.len() == 0,
        Err(_) => false,
    }
}

pub fn demo_data() -> Vec<f64> {
    // 512 samples of a noisy sine
    let n = 512;
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            (2.0 * std::f64::consts::PI * 5.0 * t).sin() + 0.15 * (2.0 * t).sin()
        })
        .collect()
}

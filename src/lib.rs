pub mod filters;
use aberth::{Complex, aberth};
use filters::{
    FilterData, NYQUIST_RATE, butterworth_filter, chebyshev_filter_1, chebyshev_filter_2,
};

const DEFAULT_ORDER: usize = 4;
const DEFAULT_RIPPLE: f64 = 5.;
const DEFAULT_ATTENUATION: f64 = 40.;
const EPSILON: f64 = 0.001;
const MAX_ITERATIONS: u32 = 10;

enum FilterType {
    BUTTERWORTH,
    CHEBYSHEV1,
    CHEBYSHEV2,
}

pub struct App {
    raw_data: Vec<f64>,
    filter: FilterType,
    cutoff_freq: f64,
    filtered_data: Option<FilterData>,
    order: usize,
    ripple: f64,
    attenuation: f64,
    poles: Option<Vec<Complex<f64>>>,
    zeros: Option<Vec<Complex<f64>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            raw_data: Vec::new(),
            filter: FilterType::BUTTERWORTH,
            cutoff_freq: filters::NYQUIST_RATE,
            filtered_data: None,
            order: DEFAULT_ORDER,
            ripple: DEFAULT_RIPPLE,
            attenuation: DEFAULT_ATTENUATION,
            poles: None,
            zeros: None,
        }
    }

    pub fn filter(&mut self) -> Result<(), String> {
        self.filtered_data = match self.filter {
            FilterType::BUTTERWORTH => {
                match butterworth_filter(&self.raw_data, self.cutoff_freq, self.order) {
                    Ok(f) => Some(f),
                    Err(e) => return Err(e),
                }
            }
            FilterType::CHEBYSHEV1 => {
                match chebyshev_filter_1(&self.raw_data, self.cutoff_freq, self.order, self.ripple)
                {
                    Ok(f) => Some(f),
                    Err(e) => return Err(e),
                }
            }
            FilterType::CHEBYSHEV2 => {
                match chebyshev_filter_1(
                    &self.raw_data,
                    self.cutoff_freq,
                    self.order,
                    self.attenuation,
                ) {
                    Ok(f) => Some(f),
                    Err(e) => return Err(e),
                }
            }
        };
        Ok(())
    }
}

fn roots_from_coeffs(coeffs: &[f64]) -> Result<Vec<aberth::Complex<f64>>, String> {
    let coeffs = coeffs.to_vec();
    coeffs.reverse();
    let roots = aberth(&coeffs, MAX_ITERATIONS, EPSILON);
    let roots_vec: Vec<aberth::Complex<f64>> = match roots.into() {
        Ok(r) => r,
        Err(_) => return Err(String::from("Faild to obtain polynomial roots")),
    };
    return roots_vec;
}

pub mod filters;
pub mod frequency;
use filters::{
    FilterData, NYQUIST_PERIOD, butterworth_filter, chebyshev_filter_1, chebyshev_filter_2,
};
use ndarray::Array2;
use ndarray_linalg::EigVals;
use num_complex::Complex;

const DEFAULT_ORDER: usize = 4;
const DEFAULT_RIPPLE: f64 = 5.;
const DEFAULT_ATTENUATION: f64 = 40.;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterType {
    #[default]
    BUTTERWORTH,
    CHEBYSHEV1,
    CHEBYSHEV2,
}

impl FilterType {
    pub const ALL: [FilterType; 3] = [
        FilterType::BUTTERWORTH,
        FilterType::CHEBYSHEV1,
        FilterType::CHEBYSHEV2,
    ];
}

impl fmt::Display for FilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FilterType::BUTTERWORTH => "Butterworth",
            FilterType::CHEBYSHEV1 => "Chebyshev I",
            FilterType::CHEBYSHEV2 => "Chebyshev II",
        };
        write!(f, "{s}")
    }
}

#[derive(Default)]
pub struct App {
    pub raw_data: Vec<f64>,
    pub filter: FilterType,
    pub cutoff_freq: f64,
    pub filtered_data: Option<FilterData>,
    pub order: usize,
    pub ripple: f64,
    pub attenuation: f64,
    pub poles: Option<Vec<Complex<f64>>>,
    pub zeros: Option<Vec<Complex<f64>>>,
    pub bode_plot: Option<Vec<f64>>,
    pub data_spectrum: Option<Vec<f64>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            raw_data: Vec::new(),
            filter: FilterType::BUTTERWORTH,
            cutoff_freq: NYQUIST_PERIOD,
            filtered_data: None,
            order: DEFAULT_ORDER,
            ripple: DEFAULT_RIPPLE,
            attenuation: DEFAULT_ATTENUATION,
            poles: None,
            zeros: None,
            bode_plot: None,
            data_spectrum: None,
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
                match chebyshev_filter_2(
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
        (self.zeros, self.poles) = match iir_zeros_poles_z(
            self.filtered_data.as_ref().unwrap().b.as_slice(),
            self.filtered_data.as_ref().unwrap().a.as_slice(),
        ) {
            Ok((z, p)) => (Some(z), Some(p)),
            Err(s) => return Err(s),
        };
        Ok(())
    }

    pub fn set_filter_type(&mut self, t: FilterType) {
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

    pub fn set_demo_data(&mut self) {
        // 512 samples of a noisy sine
        let n = 512;
        self.raw_data = (0..n)
            .map(|i| {
                let t = i as f64 / n as f64;
                (2.0 * std::f64::consts::PI * 5.0 * t).sin() + 0.15 * (2.0 * t).sin()
            })
            .collect();
    }

    pub fn fft_filtered(&mut self) -> Result<(), String> {
        if let Some(data) = filtered_data {
            self.data_spectrum = frequency::rfft_mag(data)?
        } else {
            return Err(String::from("Filtering not complete"));
        }
    }
}

/// c in ascending order: c[0] + c[1] w + ... + c[n] w^n
pub fn poly_roots_ascending_real(c_in: &[f64]) -> Result<Vec<Complex<f64>>, String> {
    if c_in.is_empty() {
        return Err("Empty polynomial".into());
    }

    // trim trailing zeros (highest degree)
    let deg = match c_in.iter().rposition(|&x| x != 0.0) {
        Some(d) => d,
        None => return Err("Zero polynomial".into()),
    };
    if deg == 0 {
        return Ok(vec![]); // constant => no roots
    }

    let lead = c_in[deg];
    let mut c = c_in[..=deg].to_vec();
    for x in &mut c {
        *x /= lead; // monic
    }

    // Companion for w^deg + a_{deg-1} w^{deg-1} + ... + a0
    let mut m = Array2::<Complex<f64>>::zeros((deg, deg));

    // first row = [-a_{deg-1}, ..., -a0]
    for j in 0..deg {
        let a = c[deg - 1 - j];
        m[(0, j)] = Complex::new(-a, 0.0);
    }
    // subdiagonal ones
    for i in 1..deg {
        m[(i, i - 1)] = Complex::new(1.0, 0.0);
    }

    let eig = m.eigvals().map_err(|e| format!("eigvals failed: {e}"))?;
    Ok(eig.to_vec())
}

/// Given filter coeffs in z^-1 form (b0..bN, a0..aM),
/// return (zeros_z, poles_z) in the z-plane.
pub fn iir_zeros_poles_z(
    b: &[f64],
    a: &[f64],
) -> Result<(Vec<Complex<f64>>, Vec<Complex<f64>>), String> {
    // Roots in w = z^-1:
    let zeros_w = poly_roots_ascending_real(b)?;
    let poles_w = poly_roots_ascending_real(a)?;

    // Convert to z = 1/w (handle w ~ 0 => z at infinity)
    let inv = |w: Complex<f64>| {
        if w.norm() == 0.0 {
            // root at w=0 => z = infinity; represent however you want
            Complex::new(f64::INFINITY, f64::INFINITY)
        } else {
            Complex::new(1.0, 0.0) / w
        }
    };

    let zeros_z: Vec<_> = zeros_w.into_iter().map(inv).collect();
    let poles_z: Vec<_> = poles_w.into_iter().map(inv).collect();
    Ok((zeros_z, poles_z))
}

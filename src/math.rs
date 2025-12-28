use sci_rs::signal::filter::{
    design::{
        DigitalFilter, FilterBandType, FilterOutputType, FilterType, Sos, SosFormatFilter,
        butter_dyn, iirfilter_dyn,
    },
    sosfiltfilt_dyn,
};
use scirs2::signal::filter;
use scirs2::fft::rfft;
use num_complex::Complex;
use ndarray::Array2;
use ndarray_linalg::EigVals;

pub const NYQUIST_PERIOD: f64 = 2.;

pub struct FilterData {
    pub filtered_data: Vec<f64>,
    pub b: Vec<f64>,
    pub a: Vec<f64>,
}

// Period in samples
pub fn cutoff_period_to_nyquist(period: f64) -> Result<f64, String> {
    if period < NYQUIST_PERIOD {
        return Err(format!(
            "Period of {period} is below the nyquist period of {NYQUIST_PERIOD}"
        ));
    }
    Ok(NYQUIST_PERIOD / period)
}

pub fn butterworth_filter(
    data: &[f64],
    cutoff_freq: f64,
    order: usize,
) -> Result<FilterData, String> {
    let (mut num, den) = match filter::butter(order, cutoff_freq, "lowpass") {
        Ok(v) => v,
        Err(_) => return Err(String::from("P-Z butterworth filter construction failed")),
    };
    normalize_lowpass_dc(&mut num, &den);
    let sos = butterworth_sos(order, vec![cutoff_freq], FilterBandType::Lowpass)?;
    let filtered = sosfiltfilt_dyn(data.iter().copied(), &sos);
    Ok(FilterData {
        filtered_data: filtered,
        b: num,
        a: den,
    })
}

pub fn chebyshev_filter_1(
    data: &[f64],
    cutoff_freq: f64,
    order: usize,
    ripple: f64,
) -> Result<FilterData, String> {
    let (mut num, den) = match filter::cheby1(order, ripple, cutoff_freq, "lowpass") {
        Ok(v) => v,
        Err(_) => return Err(String::from("Butterworth filter construction failed")),
    };
    normalize_lowpass_dc(&mut num, &den);
    let sos = chebyshev1_sos(order, vec![cutoff_freq], ripple, FilterBandType::Lowpass)?;
    let filtered = sosfiltfilt_dyn(data.iter().copied(), &sos);
    Ok(FilterData {
        filtered_data: filtered,
        b: num,
        a: den,
    })
}

pub fn chebyshev_filter_2(
    data: &[f64],
    cutoff_freq: f64,
    order: usize,
    attenuation: f64,
) -> Result<FilterData, String> {
    let (mut num, den) = match filter::cheby2(order, attenuation, cutoff_freq, "lowpass") {
        Ok(v) => v,
        Err(_) => return Err(String::from("Butterworth filter construction failed")),
    };
    normalize_lowpass_dc(&mut num, &den);
    let sos = chebyshev2_sos(
        order,
        vec![cutoff_freq],
        attenuation,
        FilterBandType::Lowpass,
    )?;
    let filtered = sosfiltfilt_dyn(data.iter().copied(), &sos);
    Ok(FilterData {
        filtered_data: filtered,
        b: num,
        a: den,
    })
}

fn butterworth_sos(
    order: usize,
    wn: Vec<f64>,
    band: FilterBandType,
    // fs: f64,
) -> Result<Vec<Sos<f64>>, String> {
    let df = butter_dyn(
        order,
        wn,
        Some(band),
        Some(false),                 // digital filter
        Some(FilterOutputType::Sos), // force SOS output
        // Some(fs),                       // wn interpreted in same units as fs
        None,
    );

    match df {
        DigitalFilter::Sos(SosFormatFilter { sos }) => Ok(sos),
        _ => Err("butter_dyn did not return SOS output".into()),
    }
}

fn chebyshev1_sos(
    order: usize,
    wn: Vec<f64>,
    ripple: f64,
    band: FilterBandType,
) -> Result<Vec<Sos<f64>>, String> {
    let df = iirfilter_dyn(
        order,
        wn,
        Some(ripple),
        None,
        Some(band),
        Some(FilterType::ChebyshevI),
        Some(false),
        Some(FilterOutputType::Sos),
        None,
    );
    match df {
        DigitalFilter::Sos(SosFormatFilter { sos }) => Ok(sos),
        _ => Err("iirfilter_dyn did not return SOS output".into()),
    }
}

fn chebyshev2_sos(
    order: usize,
    wn: Vec<f64>,
    attenuation: f64,
    band: FilterBandType,
) -> Result<Vec<Sos<f64>>, String> {
    let df = iirfilter_dyn(
        order,
        wn,
        None,
        Some(attenuation),
        Some(band),
        Some(FilterType::ChebyshevII),
        Some(false),
        Some(FilterOutputType::Sos),
        None,
    );
    match df {
        DigitalFilter::Sos(SosFormatFilter { sos }) => Ok(sos),
        _ => Err("iirfilter_dyn did not return SOS output".into()),
    }
}

fn normalize_lowpass_dc(b: &mut [f64], a: &[f64]) {
    let sum_b: f64 = b.iter().sum();
    let sum_a: f64 = a.iter().sum();
    let g = sum_b / sum_a; // H(0)
    for bi in b.iter_mut() {
        *bi /= g; // make H(0) = 1
    }
}

pub fn rfft_mag(data: &[f64]) -> Result<Vec<f64>, String> {
    let output = match rfft(data, None) {
        Ok(r) => r,
        Err(_) => return Err(String::from("Could not take fft of data")),
    };
    Ok(output.into_iter().map(|x| x.norm()).collect())
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

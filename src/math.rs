use core::cmp::min;
use ndarray::Array2;
use ndarray_linalg::EigVals;
use num_complex::Complex;
use sci_rs::signal::filter::{
    design::{
        DigitalFilter, FilterBandType, FilterOutputType, FilterType, Sos, SosFormatFilter,
        butter_dyn, iirfilter_dyn,
    },
    sosfiltfilt_dyn,
};
use scirs2::fft::rfft;
use scirs2::signal::filter;

type PzTuple = (Vec<Complex<f64>>, Vec<Complex<f64>>);

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
    let min_cnt = min_len_for_sosfiltfilt(&sos);
    if data.len() < min_cnt {
        return Err(format!(
            "Requires {} points for filtering. Got {}",
            min_cnt,
            data.len()
        ));
    }
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
) -> Result<Vec<Sos<f64>>, String> {
    let df = butter_dyn(
        order,
        wn,
        Some(band),
        Some(false),
        Some(FilterOutputType::Sos),
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
        *bi /= g; // force H(0) = 1 (unity DC gain)
    }
}

pub fn rfft_mag(data: &[f64]) -> Result<Vec<f64>, String> {
    let output = match rfft(data, None) {
        Ok(r) => r,
        Err(_) => return Err(String::from("Could not take fft of data")),
    };
    Ok(output.into_iter().map(|x| x.norm()).collect())
}

// c in ascending order
pub fn poly_roots_ascending_real(c_in: &[f64]) -> Result<Vec<Complex<f64>>, String> {
    if c_in.is_empty() {
        return Err("Empty polynomial".into());
    }

    // trim trailing zeros
    let deg = match c_in.iter().rposition(|&x| x != 0.0) {
        Some(d) => d,
        None => return Err("Zero polynomial".into()),
    };
    if deg == 0 {
        return Ok(vec![]); // constant so no roots
    }

    let lead = c_in[deg];
    let mut c = c_in[..=deg].to_vec();
    for x in &mut c {
        *x /= lead; // monic
    }

    let mut m = Array2::<Complex<f64>>::zeros((deg, deg));

    for j in 0..deg {
        let a = c[deg - 1 - j];
        m[(0, j)] = Complex::new(-a, 0.0);
    }
    for i in 1..deg {
        m[(i, i - 1)] = Complex::new(1.0, 0.0);
    }

    let eig = m.eigvals().map_err(|e| format!("eigvals failed: {e}"))?;
    Ok(eig.to_vec())
}

pub fn iir_zeros_poles_z(b: &[f64], a: &[f64]) -> Result<PzTuple, String> {
    let zeros_w = poly_roots_ascending_real(b)?;
    let poles_w = poly_roots_ascending_real(a)?;

    let inv = |w: Complex<f64>| {
        if w.norm() == 0.0 {
            Complex::new(f64::INFINITY, f64::INFINITY)
        } else {
            Complex::new(1.0, 0.0) / w
        }
    };

    let zeros_z: Vec<_> = zeros_w.into_iter().map(inv).collect();
    let poles_z: Vec<_> = poles_w.into_iter().map(inv).collect();
    Ok((zeros_z, poles_z))
}

pub fn bode_mag_logspace(b: &[f64], a: &[f64], fs: f64, n_points: usize) -> (Vec<f64>, Vec<f64>) {
    let n_points = n_points.max(16);

    let f_min = (fs * 1e-4).max(1e-9);
    let f_max = (fs * 0.5).max(f_min * 10.0);

    let log_fmin = f_min.ln();
    let log_fmax = f_max.ln();

    let mut freqs = Vec::with_capacity(n_points);
    let mut mags = Vec::with_capacity(n_points);

    for i in 0..n_points {
        let t = i as f64 / (n_points - 1) as f64;
        let f = (log_fmin + t * (log_fmax - log_fmin)).exp();
        let omega = 2.0 * std::f64::consts::PI * (f / fs); // rad/sample

        let (c, s) = (omega.cos(), omega.sin());

        let (mut zr, mut zi) = (1.0_f64, 0.0_f64);

        let mut num_r = 0.0_f64;
        let mut num_i = 0.0_f64;
        for &bk in b {
            num_r += bk * zr;
            num_i += bk * zi;
            let new_zr = zr * c + zi * s;
            let new_zi = zi * c - zr * s;
            zr = new_zr;
            zi = new_zi;
        }

        let (mut zr, mut zi) = (1.0_f64, 0.0_f64);
        let mut den_r = 0.0_f64;
        let mut den_i = 0.0_f64;
        for &ak in a {
            den_r += ak * zr;
            den_i += ak * zi;
            let new_zr = zr * c + zi * s;
            let new_zi = zi * c - zr * s;
            zr = new_zr;
            zi = new_zi;
        }

        // H = num/den
        let den_mag2 = den_r * den_r + den_i * den_i;
        let mag = if den_mag2 > 0.0 {
            let h_r = (num_r * den_r + num_i * den_i) / den_mag2;
            let h_i = (num_i * den_r - num_r * den_i) / den_mag2;
            (h_r * h_r + h_i * h_i).sqrt()
        } else {
            f64::NAN
        };

        freqs.push(f);
        mags.push(mag);
    }

    (freqs, mags)
}

fn min_len_for_sosfiltfilt<
    F: Copy + PartialEq + rustfft::num_traits::Zero + sci_rs::na::RealField,
>(
    sos: &[Sos<F>],
) -> usize {
    let n = sos.len();
    let mut ntaps = 2 * n + 1;

    let bzeros = sos.iter().filter(|s| s.b[2] == F::zero()).count();
    let azeros = sos.iter().filter(|s| s.a[2] == F::zero()).count();
    ntaps -= min(bzeros, azeros);

    let edge = 3 * ntaps;
    edge + 1
}

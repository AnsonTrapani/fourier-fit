use sci_rs::signal::filter::{
    design::{
        DigitalFilter, FilterBandType, FilterOutputType, FilterType, Sos, SosFormatFilter,
        butter_dyn, iirfilter_dyn,
    },
    sosfiltfilt_dyn,
};
use scirs2::signal::filter;

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

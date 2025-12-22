use sci_rs::signal::filter::{
    design::{DigitalFilter, FilterBandType, FilterOutputType, Sos, SosFormatFilter, butter_dyn},
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
    let (num, den) = match filter::butter(order, cutoff_freq, "lowpass") {
        Ok(v) => v,
        Err(_) => return Err(String::from("P-Z butterworth filter construction failed")),
    };
    let sos = match butterworth_sos(order, vec![cutoff_freq], FilterBandType::Lowpass) {
        Ok(v) => v,
        Err(_) => return Err(String::from("Butterworth filter construction failed")),
    };
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
    let (num, den) = match filter::cheby1(order, ripple, cutoff_freq, "lowpass") {
        Ok(v) => v,
        Err(_) => return Err(String::from("Butterworth filter construction failed")),
    };
    let filtered = match filter::filtfilt(&num, &den, data) {
        Ok(f) => f,
        Err(_) => return Err(String::from("Butterworth filtering failed")),
    };
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
    let (num, den) = match filter::cheby2(order, attenuation, cutoff_freq, "lowpass") {
        Ok(v) => v,
        Err(_) => return Err(String::from("Butterworth filter construction failed")),
    };
    let filtered = match filter::filtfilt(&num, &den, data) {
        Ok(f) => f,
        Err(_) => return Err(String::from("Butterworth filtering failed")),
    };
    Ok(FilterData {
        filtered_data: filtered,
        b: num,
        a: den,
    })
}

pub fn butterworth_sos(
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

use scirs2::signal::filter;

pub const NYQUIST_RATE: f64 = 0.5; // In cycles per sample
const NYQUIST_PERIOD: f64 = 1. / NYQUIST_RATE;

pub struct FilterData {
    filtered_data: Vec<f64>,
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
    Ok(1. / period / NYQUIST_RATE)
}

// Freq in cycles per sample
pub fn cutoff_freq_to_nyquist(freq: f64) -> Result<f64, String> {
    if freq > NYQUIST_RATE {
        return Err(format!(
            "Period of {freq} is greater than the nyquist period of {NYQUIST_RATE}"
        ));
    }
    Ok(freq / NYQUIST_RATE)
}

pub fn butterworth_filter(
    data: &[f64],
    cutoff_freq: f64,
    order: usize,
) -> Result<FilterData, String> {
    let (num, den) = match filter::butter(order, cutoff_freq, "lowpass") {
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

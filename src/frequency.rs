use scirs2::fft::rfft;

pub fn rfft_mag(data: &[f64]) -> Result<Vec<f64>, String> {
    let output = match rfft(data, None) {
        Ok(r) => r,
        Err(_) => String::from("Could not take fft of data"),
    };
    Ok(output.map(|x| x.norm()).collect())
}

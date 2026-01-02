#[derive(Clone, Copy, Debug)]
pub struct Candle {
    pub t: f64, // time index (seconds, days, sample index... anything monotonic)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn vec_to_candles(data: &[f64], num_per_candle: usize) -> Result<Vec<Candle>, String> {
    if num_per_candle == 0 {
        return Err("Cannot have a chunk size of zero in candle making function".into());
    }
    let mut candles: Vec<Candle> =
        Vec::with_capacity((data.len() as f64 / num_per_candle as f64).ceil() as usize);
    let chunks: Vec<&[f64]> = (0..data.len())
        .step_by(num_per_candle)
        .map(|i| &data[i..(i + num_per_candle + 1).min(data.len())])
        .collect();
    for (i, &chunk) in chunks.iter().enumerate() {
        candles.push(Candle {
            t: i as f64,
            open: chunk[0],
            close: chunk[chunk.len() - 1],
            high: chunk
                .iter()
                .fold(f64::NEG_INFINITY, |prev, curr| prev.max(*curr)),
            low: chunk
                .iter()
                .fold(f64::INFINITY, |prev, curr| prev.min(*curr)),
        })
    }
    Ok(candles)
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandleLengths {
    #[default]
    Weekly,
    Monthly,
    Yearly,
}

impl From<CandleLengths> for usize {
    fn from(value: CandleLengths) -> Self {
        match value {
            CandleLengths::Weekly => 7,
            CandleLengths::Monthly => 30,
            CandleLengths::Yearly => 365,
        }
    }
}

impl std::fmt::Display for CandleLengths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CandleLengths::Weekly => "Weekly",
            CandleLengths::Monthly => "Monthly",
            CandleLengths::Yearly => "Yearly",
        };
        write!(f, "{s}")
    }
}

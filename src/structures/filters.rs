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

impl std::fmt::Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FilterType::BUTTERWORTH => "Butterworth",
            FilterType::CHEBYSHEV1 => "Chebyshev I",
            FilterType::CHEBYSHEV2 => "Chebyshev II",
        };
        write!(f, "{s}")
    }
}

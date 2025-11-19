/// Descriptive statistics results
#[derive(Debug, Clone)]
pub struct DescriptiveStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub mode: Vec<f64>,
    pub std_dev: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub cv: f64,
    pub mad: f64,
    pub confidence_intervals: Option<ConfidenceIntervals>,
    pub robust_cv: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ConfidenceIntervals {
    pub mean_ci: (f64, f64),
    pub median_ci: (f64, f64),
    pub std_dev_ci: (f64, f64),
}
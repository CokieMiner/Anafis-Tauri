#[derive(Debug, Clone)]
pub struct CorrelationTestResult {
    pub method: String,
    pub variable_1: usize,
    pub variable_2: usize,
    pub correlation: f64,
    pub statistic: f64,
    pub p_value: f64,
    pub significant: bool,
}

#[derive(Debug, Clone)]
pub struct CorrelationAnalysis {
    pub matrix: Vec<Vec<f64>>,
    pub methods: Vec<String>,
    pub significance_tests: Vec<CorrelationTestResult>,
}
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::SpecialFunctions;
use statrs::distribution::ContinuousCDF;
use crate::scientific::statistics::types::DistributionFit;
use super::super::goodness_of_fit::goodness_of_fit;

/// Fit Student's t distribution
pub fn fit_students_t_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    // Use sample kurtosis to estimate degrees of freedom
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1.0);

    if variance <= 0.0 {
        return Err("Student's t fit undefined: zero variance (constant data)".to_string());
    }

    let std_dev = variance.sqrt();

    let kurtosis = data.iter()
        .map(|x| ((x - mean) / std_dev).powi(4))
        .sum::<f64>() / n - 3.0; // Excess kurtosis

    // For t-distribution: kurtosis = 6/(ν-4) for ν > 4
    let nu = if kurtosis > 0.0 {
        6.0 / kurtosis + 4.0
    } else {
        30.0 // Default for light-tailed data
    };

    if nu <= 2.0 || nu.is_nan() {
        return Err("Student's t fit undefined: invalid degrees of freedom".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // t-distribution PDF (corrected formula)
            let t = (x - mean) / std_dev;
            let constant = SpecialFunctions::gamma((nu + 1.0) / 2.0) / (std_dev * (nu * std::f64::consts::PI).sqrt() * SpecialFunctions::gamma(nu / 2.0));
            let pdf = constant * (1.0 + t.powi(2) / nu).powf(-(nu + 1.0) / 2.0);
            pdf.ln()
        })
        .sum::<f64>();

    let k = 3.0; // parameters: location, scale, df
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * n.ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "student_t".to_string(),
        parameters: vec![
            ("location".to_string(), mean),
            ("scale".to_string(), std_dev),
            ("df".to_string(), nu),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| student_t_cdf(x, mean, std_dev, nu))?,
    })
}

/// Fit Cauchy distribution (heavy-tailed)
pub fn fit_cauchy_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    // Use median and MAD as robust estimators
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let median = if sorted.len().is_multiple_of(2) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };

    // MAD (Median Absolute Deviation)
    let deviations: Vec<f64> = data.iter().map(|x| (x - median).abs()).collect();
    let mut sorted_dev = deviations.clone();
    sorted_dev.sort_by(|a, b| a.total_cmp(b));
    let mad = if sorted_dev.len().is_multiple_of(2) {
        (sorted_dev[sorted_dev.len() / 2 - 1] + sorted_dev[sorted_dev.len() / 2]) / 2.0
    } else {
        sorted_dev[sorted_dev.len() / 2]
    };

    let scale = mad / 0.6745; // Convert MAD to scale parameter

    if scale <= 0.0 || scale.is_nan() {
        return Err("Cauchy fit undefined: invalid scale parameter".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Cauchy PDF: 1/(πγ(1 + ((x-x₀)/γ)²))
            let pdf = 1.0 / (std::f64::consts::PI * scale * (1.0 + ((x - median) / scale).powi(2)));
            pdf.ln()
        })
        .sum::<f64>();

    let k = 2.0; // parameters: location, scale
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * (data.len() as f64).ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "cauchy".to_string(),
        parameters: vec![
            ("location".to_string(), median),
            ("scale".to_string(), scale),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| cauchy_cdf(x, median, scale))?,
    })
}

/// Student's t CDF using statrs
fn student_t_cdf(x: f64, location: f64, scale: f64, df: f64) -> Result<f64, String> {
    // Standardize to location 0, scale 1
    let standardized = (x - location) / scale;

    // Use statrs StudentsT CDF
    let t_dist = statrs::distribution::StudentsT::new(0.0, 1.0, df)
        .map_err(|e| format!("Failed to create StudentsT distribution: {}", e))?;
    Ok(t_dist.cdf(standardized))
}

/// Cauchy CDF
fn cauchy_cdf(x: f64, location: f64, scale: f64) -> Result<f64, String> {
    Ok(0.5 + (1.0 / std::f64::consts::PI) * ((x - location) / scale).atan())
}
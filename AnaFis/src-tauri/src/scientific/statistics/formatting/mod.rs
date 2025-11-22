//! Output formatting module for statistics_2
//!
//! This module provides structured formatting for statistical analysis results,
//! including precision control, sanitization, and various output formats.

use std::fmt;
use std::collections::HashMap;

/// Descriptive statistics data structure
#[derive(Debug, Clone)]
pub struct DescriptiveStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub mode: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub iqr: f64,
}

/// Configuration for output formatting
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Number of decimal places for floating point numbers
    pub precision: usize,
    /// Whether to use scientific notation for very small/large numbers
    pub scientific_notation: bool,
    /// Threshold for switching to scientific notation
    pub scientific_threshold: f64,
    /// Whether to include confidence intervals in output
    pub include_confidence_intervals: bool,
    /// Confidence level for intervals (default 0.95)
    pub confidence_level: f64,
    /// Whether to sanitize NaN and infinite values
    pub sanitize_values: bool,
    /// Custom labels for different statistics
    pub custom_labels: HashMap<String, String>,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            precision: 4,
            scientific_notation: true,
            scientific_threshold: 1e-3,
            include_confidence_intervals: true,
            confidence_level: 0.95,
            sanitize_values: true,
            custom_labels: HashMap::new(),
        }
    }
}

/// Formatted statistical result
#[derive(Debug, Clone)]
pub struct FormattedResult {
    /// The formatted value as a string
    pub value: String,
    /// Optional confidence interval
    pub confidence_interval: Option<(String, String)>,
    /// The statistical test or measure name
    pub label: String,
    /// Units or additional context
    pub units: Option<String>,
    /// Significance level (p-value interpretation)
    pub significance: Option<String>,
}

/// Main formatting engine
pub struct OutputFormatter {
    config: FormatConfig,
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter {
    /// Create a new formatter with default configuration
    pub fn new() -> Self {
        Self {
            config: FormatConfig::default(),
        }
    }

    /// Create a new formatter with custom configuration
    pub fn with_config(config: FormatConfig) -> Self {
        Self { config }
    }

    /// Format a single floating point value
    pub fn format_value(&self, value: f64) -> String {
        if self.config.sanitize_values && (!value.is_finite()) {
            return if value.is_nan() {
                "NaN".to_string()
            } else if value.is_infinite() && value.is_sign_positive() {
                "∞".to_string()
            } else {
                "-∞".to_string()
            };
        }

        if self.config.scientific_notation &&
           (value.abs() < self.config.scientific_threshold ||
            value.abs() > 1.0 / self.config.scientific_threshold) {
            format!("{:.precision$e}", value, precision = self.config.precision)
        } else {
            format!("{:.precision$}", value, precision = self.config.precision)
        }
    }

    /// Format a p-value with significance indicators
    pub fn format_p_value(&self, p_value: f64) -> String {
        let formatted = self.format_value(p_value);
        let significance = if p_value < 0.001 {
            "***"
        } else if p_value < 0.01 {
            "**"
        } else if p_value < 0.05 {
            "*"
        } else if p_value < 0.1 {
            "."
        } else {
            ""
        };

        format!("{} {}", formatted, significance)
    }

    /// Format a confidence interval
    pub fn format_confidence_interval(&self, lower: f64, upper: f64) -> (String, String) {
        (self.format_value(lower), self.format_value(upper))
    }

    /// Format a statistical result with full context
    pub fn format_result(
        &self,
        value: f64,
        label: &str,
        units: Option<&str>,
        confidence_interval: Option<(f64, f64)>,
        p_value: Option<f64>,
    ) -> FormattedResult {
        let formatted_value = self.format_value(value);
        let formatted_ci = confidence_interval.map(|(l, u)| self.format_confidence_interval(l, u));
        let significance = p_value.map(|p| self.interpret_significance(p));

        let display_label = self.config.custom_labels.get(label)
            .cloned()
            .unwrap_or_else(|| label.to_string());

        FormattedResult {
            value: formatted_value,
            confidence_interval: formatted_ci,
            label: display_label,
            units: units.map(|u| u.to_string()),
            significance,
        }
    }

    /// Interpret significance level
    fn interpret_significance(&self, p_value: f64) -> String {
        if p_value < 0.001 {
            "Highly significant".to_string()
        } else if p_value < 0.01 {
            "Very significant".to_string()
        } else if p_value < 0.05 {
            "Significant".to_string()
        } else if p_value < 0.1 {
            "Marginally significant".to_string()
        } else {
            "Not significant".to_string()
        }
    }

    /// Format descriptive statistics
    pub fn format_descriptive_stats(&self, stats: &DescriptiveStats) -> String {
        let mut output = String::new();
        output.push_str("Descriptive Statistics:\n");
        output.push_str(&format!("  {}: {}\n",
            self.get_label("count"), self.format_value(stats.count as f64)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("mean"), self.format_value(stats.mean)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("median"), self.format_value(stats.median)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("mode"), self.format_value(stats.mode)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("std_dev"), self.format_value(stats.std_dev)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("variance"), self.format_value(stats.variance)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("skewness"), self.format_value(stats.skewness)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("kurtosis"), self.format_value(stats.kurtosis)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("min"), self.format_value(stats.min)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("max"), self.format_value(stats.max)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("range"), self.format_value(stats.range)));
        output.push_str(&format!("  {}: {}\n",
            self.get_label("iqr"), self.format_value(stats.iqr)));
        output
    }

    /// Get label for a statistic, with fallback to default
    fn get_label(&self, key: &str) -> String {
        self.config.custom_labels.get(key)
            .cloned()
            .unwrap_or_else(|| match key {
                "count" => "Count",
                "mean" => "Mean",
                "median" => "Median",
                "mode" => "Mode",
                "std_dev" => "Standard Deviation",
                "variance" => "Variance",
                "skewness" => "Skewness",
                "kurtosis" => "Kurtosis",
                "min" => "Minimum",
                "max" => "Maximum",
                "range" => "Range",
                "iqr" => "Interquartile Range",
                _ => key,
            }.to_string())
    }

    /// Format hypothesis test results
    pub fn format_hypothesis_test(
        &self,
        test_name: &str,
        statistic: f64,
        p_value: f64,
        confidence_interval: Option<(f64, f64)>,
        effect_size: Option<f64>,
    ) -> String {
        let mut output = format!("{} Test Results:\n", test_name);
        output.push_str(&format!("  Test Statistic: {}\n", self.format_value(statistic)));
        output.push_str(&format!("  p-value: {}\n", self.format_p_value(p_value)));

        if let Some(ci) = confidence_interval {
            let (lower, upper) = self.format_confidence_interval(ci.0, ci.1);
            output.push_str(&format!("  {:.0}% Confidence Interval: ({}, {})\n",
                self.config.confidence_level * 100.0, lower, upper));
        }

        if let Some(es) = effect_size {
            output.push_str(&format!("  Effect Size: {}\n", self.format_value(es)));
        }

        output.push_str(&format!("  Significance: {}\n", self.interpret_significance(p_value)));
        output
    }

    /// Format correlation results
    pub fn format_correlation(
        &self,
        r: f64,
        p_value: f64,
        confidence_interval: Option<(f64, f64)>,
        n: usize,
    ) -> String {
        let mut output = "Correlation Analysis:\n".to_string();
        output.push_str(&format!("  Pearson r: {}\n", self.format_value(r)));
        output.push_str(&format!("  Sample Size: {}\n", n));
        output.push_str(&format!("  p-value: {}\n", self.format_p_value(p_value)));

        if let Some(ci) = confidence_interval {
            let (lower, upper) = self.format_confidence_interval(ci.0, ci.1);
            output.push_str(&format!("  {:.0}% CI: ({}, {})\n",
                self.config.confidence_level * 100.0, lower, upper));
        }

        output.push_str(&format!("  Strength: {}\n", self.interpret_correlation_strength(r)));
        output
    }

    /// Interpret correlation strength
    fn interpret_correlation_strength(&self, r: f64) -> &'static str {
        let abs_r = r.abs();
        if abs_r >= 0.8 {
            "Very strong"
        } else if abs_r >= 0.6 {
            "Strong"
        } else if abs_r >= 0.4 {
            "Moderate"
        } else if abs_r >= 0.2 {
            "Weak"
        } else {
            "Very weak"
        }
    }

    /// Format ANOVA results
    pub fn format_anova(
        &self,
        f_statistic: f64,
        p_value: f64,
        df_between: f64,
        df_within: f64,
        eta_squared: Option<f64>,
    ) -> String {
        let mut output = "ANOVA Results:\n".to_string();
        output.push_str(&format!("  F({:.1}, {:.1}) = {}\n",
            df_between, df_within, self.format_value(f_statistic)));
        output.push_str(&format!("  p-value: {}\n", self.format_p_value(p_value)));

        if let Some(eta2) = eta_squared {
            output.push_str(&format!("  η² = {}\n", self.format_value(eta2)));
            output.push_str(&format!("  Effect Size: {}\n", self.interpret_eta_squared(eta2)));
        }

        output
    }

    /// Interpret eta squared effect size
    fn interpret_eta_squared(&self, eta2: f64) -> &'static str {
        if eta2 >= 0.14 {
            "Large"
        } else if eta2 >= 0.06 {
            "Medium"
        } else if eta2 >= 0.01 {
            "Small"
        } else {
            "Negligible"
        }
    }

    /// Format regression results
    pub fn format_regression(
        &self,
        r_squared: f64,
        f_statistic: f64,
        p_value: f64,
        coefficients: &[(String, f64, f64, f64)], // name, coef, se, t_stat
        intercept: Option<(f64, f64, f64)>, // intercept, se, t_stat
    ) -> String {
        let mut output = "Regression Results:\n".to_string();
        output.push_str(&format!("  R² = {}\n", self.format_value(r_squared)));
        output.push_str(&format!("  F-statistic: {}\n", self.format_value(f_statistic)));
        output.push_str(&format!("  p-value: {}\n", self.format_p_value(p_value)));
        output.push_str("\nCoefficients:\n");

        if let Some((int, int_se, int_t)) = intercept {
            output.push_str(&format!("  Intercept: {} (SE: {}, t: {})\n",
                self.format_value(int), self.format_value(int_se), self.format_value(int_t)));
        }

        for (name, coef, se, t_stat) in coefficients {
            output.push_str(&format!("  {}: {} (SE: {}, t: {})\n",
                name, self.format_value(*coef), self.format_value(*se), self.format_value(*t_stat)));
        }

        output
    }

    /// Format quality control results
    pub fn format_quality_control(
        &self,
        control_limits: &crate::scientific::statistics::quality_control::ControlLimits,
        violations: &[String],
        capability_indices: &crate::scientific::statistics::quality_control::ProcessCapability,
    ) -> String {
        let mut output = "Quality Control Analysis:\n".to_string();
        output.push_str("Control Limits:\n");
        output.push_str(&format!("  UCL: {}\n", self.format_value(control_limits.upper_control_limit)));
        output.push_str(&format!("  Center: {}\n", self.format_value(control_limits.center_line)));
        output.push_str(&format!("  LCL: {}\n", self.format_value(control_limits.lower_control_limit)));

        output.push_str("\nCapability Indices:\n");
        output.push_str(&format!("  Cp: {}\n", self.format_value(capability_indices.cp)));
        output.push_str(&format!("  Cpk: {}\n", self.format_value(capability_indices.cpk)));
        output.push_str(&format!("  PPM Defective: {}\n", self.format_value(capability_indices.ppm_defective)));
        output.push_str(&format!("  Assessment: {}\n", capability_indices.capability_assessment));
        output.push_str(&format!("  Cpk Interpretation: {}\n", capability_indices.cpk_interpretation));

        if !violations.is_empty() {
            output.push_str(&format!("\nViolations Detected: {}\n", violations.len()));
            for (i, violation) in violations.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, violation));
            }
        } else {
            output.push_str("\nNo violations detected - process appears stable.\n");
        }

        output
    }

    /// Format reliability analysis results
    pub fn format_reliability(
        &self,
        cronbach_alpha: f64,
        mcdonald_omega: Option<f64>,
        item_statistics: &[(String, f64, f64)], // name, item_total_corr, alpha_if_deleted
        confidence_interval: Option<(f64, f64)>,
    ) -> String {
        let mut output = "Reliability Analysis:\n".to_string();
        output.push_str(&format!("  Cronbach's α: {}\n", self.format_value(cronbach_alpha)));

        if let Some(omega) = mcdonald_omega {
            output.push_str(&format!("  McDonald's ω: {}\n", self.format_value(omega)));
        }

        if let Some((lower, upper)) = confidence_interval {
            let (l_fmt, u_fmt) = self.format_confidence_interval(lower, upper);
            output.push_str(&format!("  {:.0}% CI: ({}, {})\n",
                self.config.confidence_level * 100.0, l_fmt, u_fmt));
        }

        output.push_str(&format!("  Reliability: {}\n", self.interpret_reliability(cronbach_alpha)));

        output.push_str("\nItem Statistics:\n");
        for (name, corr, alpha_del) in item_statistics {
            output.push_str(&format!("  {}: r_it = {}, α_if_del = {}\n",
                name, self.format_value(*corr), self.format_value(*alpha_del)));
        }

        output
    }

    /// Interpret reliability coefficient
    fn interpret_reliability(&self, alpha: f64) -> &'static str {
        if alpha >= 0.9 {
            "Excellent"
        } else if alpha >= 0.8 {
            "Good"
        } else if alpha >= 0.7 {
            "Acceptable"
        } else if alpha >= 0.6 {
            "Questionable"
        } else if alpha >= 0.5 {
            "Poor"
        } else {
            "Unacceptable"
        }
    }

    /// Format power analysis results
    pub fn format_power_analysis(
        &self,
        power: f64,
        sample_size: usize,
        effect_size: f64,
        alpha: f64,
        test_type: &str,
    ) -> String {
        let mut output = format!("Power Analysis ({}):\n", test_type);
        output.push_str(&format!("  Power (1-β): {}\n", self.format_value(power)));
        output.push_str(&format!("  Sample Size: {}\n", sample_size));
        output.push_str(&format!("  Effect Size: {}\n", self.format_value(effect_size)));
        output.push_str(&format!("  α: {}\n", self.format_value(alpha)));
        output.push_str(&format!("  Power Level: {}\n", self.interpret_power(power)));
        output
    }

    /// Interpret power level
    fn interpret_power(&self, power: f64) -> &'static str {
        if power >= 0.8 {
            "Adequate"
        } else if power >= 0.7 {
            "Moderate"
        } else if power >= 0.5 {
            "Low"
        } else {
            "Very low"
        }
    }

    /// Format visualization suggestions
    pub fn format_visualization_suggestions(
        &self,
        suggestions: &[crate::scientific::statistics::visualization::VisualizationSuggestions],
    ) -> String {
        let mut output = "Visualization Suggestions:\n".to_string();

        for suggestion in suggestions {
            for (i, plot) in suggestion.recommended_plots.iter().enumerate() {
                output.push_str(&format!("{}. {} - {}\n",
                    i + 1, plot.plot_type, plot.description));
                output.push_str(&format!("   Priority: {:?}\n", plot.priority));
            }
        }

        output
    }
}

impl fmt::Display for FormattedResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.label, self.value)?;

        if let Some(units) = &self.units {
            write!(f, " {}", units)?;
        }

        if let Some((lower, upper)) = &self.confidence_interval {
            write!(f, " (95% CI: {}, {})", lower, upper)?;
        }

        if let Some(sig) = &self.significance {
            write!(f, " [{}]", sig)?;
        }

        Ok(())
    }
}
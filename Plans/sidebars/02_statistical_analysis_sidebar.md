# Statistical Analysis Sidebar 📊

**Status**: Planned  
**Priority**: High  
**Complexity**: High  
**Dependencies**: statrs, ndarray, nalgebra, rand, ECharts

---

## Igor Pro vs Python vs R vs AnaFis Statistical Analysis Comparison

Based on the capabilities of Igor Pro, Python scientific stack, and R statistical ecosystem, here's how AnaFis compares:

### ❌ Major Gaps (High Priority)

- ✓ Hypothesis tests have correct p-values with proper precision reporting ✅ IMPLEMENTED
- ✓ Effect sizes included with all relevant test results ✅ IMPLEMENTED
- ✓ Power analysis with accurate non-central distribution approximations ✅ IMPLEMENTED

2. **Regression Analysis**
   - **Igor Pro**: Curve fitting with GUI, extensive built-in functions
   - **Python**: scikit-learn, statsmodels - machine learning and statistical modeling
   - **R**: lm(), glm(), nlme - most sophisticated statistical modeling
   - **AnaFis**: ❌ NOT IMPLEMENTED (separate sidebar planned)

3. **Data Management & Manipulation**
   - **Igor Pro**: Wave data structures, powerful data manipulation
   - **Python**: pandas, NumPy - industry standard data manipulation
   - **R**: dplyr, data.table - most elegant data manipulation syntax
   - **AnaFis**: ⚠️ PARTIALLY IMPLEMENTED (basic NaN handling only)

### ⚠️ Significant Gaps (Medium Priority)

4. **Advanced Plotting & Visualization**
   - **Igor Pro**: Publication-quality 2D/3D plotting, most intuitive GUI
   - **Python**: matplotlib, seaborn, plotly - highly customizable
   - **R**: ggplot2, lattice - most statistically sophisticated plotting
   - **AnaFis**: ⚠️ BASIC (ECharts integration, needs fitting overlays)

5. **Programming & Automation**
   - **Igor Pro**: Built-in programming language, macro system
   - **Python**: Full programming language, extensive ecosystem
   - **R**: Statistical programming, functional paradigm
   - **AnaFis**: ❌ NOT IMPLEMENTED (no scripting interface)

6. **Multivariate Analysis**
   - **Igor Pro**: PCA, factor analysis, multivariate fitting
   - **Python**: scikit-learn, statsmodels - machine learning focused
   - **R**: psych, FactoMineR - most comprehensive statistical methods
   - **AnaFis**: ❌ NOT IMPLEMENTED (separate sidebar planned)

### 🔧 Infrastructure Gaps (Technical)

7. **Package/Library Ecosystem**
   - **Igor Pro**: Closed ecosystem, limited third-party extensions
   - **Python**: Massive ecosystem, community-driven development
   - **R**: CRAN with 18,000+ packages, most specialized statistical tools
   - **AnaFis**: ⚠️ BASIC (growing Rust ecosystem, limited compared to Python/R)

8. **Performance & Scalability**
   - **Igor Pro**: Excellent for medium datasets, GUI can slow large operations
   - **Python**: Excellent performance with NumPy, scales well
   - **R**: Good for statistics, memory limitations with large datasets
   - **AnaFis**: ✅ STRONG (Rust backend, better performance than Python/R for numerical computing)

9. **File Format Support**
   - **Igor Pro**: Proprietary formats, good instrument integration
   - **Python**: Supports virtually all formats via libraries
   - **R**: Excellent format support, especially statistical formats
   - **AnaFis**: ⚠️ BASIC (CSV, JSON, Parquet, Excel - needs expansion)

### ✅ AnaFis Competitive Advantages

**Built-in Uncertainty Quantification**
- **AnaFis**: ✅ Native uncertainty propagation, bootstrap CIs
- **Igor Pro**: ⚠️ Limited uncertainty handling
- **Python**: ⚠️ Requires manual implementation (uncertainties library)
- **R**: ⚠️ Requires packages (propagate, errors) - not built-in

**Spreadsheet Integration**
- **AnaFis**: ✅ Native spreadsheet interface, real-time analysis
- **Igor Pro**: ⚠️ Separate application, import/export workflow
- **Python**: ⚠️ Requires Jupyter or separate tools (pandas, xlwings)
- **R**: ⚠️ Requires RStudio or external tools (openxlsx, readxl)

**User Experience for Non-Programmers**
- **AnaFis**: ✅ Drag-and-drop, GUI-first, progressive disclosure
- **Igor Pro**: ⚠️ GUI with learning curve, requires some programming
- **Python**: ❌ Steep learning curve, requires programming knowledge
- **R**: ❌ Requires statistical programming knowledge

**Cost & Accessibility**
- **AnaFis**: ✅ Free, open-source
- **Igor Pro**: ❌ Expensive commercial license (~$1,000+)
- **Python**: ✅ Free, open-source ecosystem
- **R**: ✅ Free, open-source

### 🎯 Competitive Positioning

| Category | Igor Pro | Python | R | AnaFis |
|----------|----------|--------|----|--------|
| **Ease of Use** | High | Low | Medium | Highest |
| **Uncertainty Quantification** | Low | Medium | Medium | Highest |
| **Spreadsheet Integration** | Low | Medium | Low | Highest |
| **Statistical Depth** | High | High | Highest | Medium |
| **Programming Required** | Medium | High | High | None |
| **Cost** | High | Free | Free | Free |
| **Performance** | High | High | Medium | High |
| **Visualization Quality** | Highest | High | High | Medium |

### 💡 Strategic Market Positioning

**AnaFis Target Users**: Experimental scientists and engineers who need:
- Uncertainty-aware data analysis
- Spreadsheet-like workflow
- Professional results without programming
- Integration with measurement instruments

**Key Differentiators**:
1. **Uncertainty-First Design**: Built-in error propagation vs. afterthought in others
2. **Spreadsheet Native**: No export/import friction
3. **Performance**: Rust backend faster than Python/R for numerical work
4. **Modern UX**: Intuitive interface vs. programming-heavy alternatives

**Competition Analysis**:
- **vs Igor Pro**: Free alternative with better uncertainty handling, modern interface
- **vs Python**: GUI-first approach for non-programmers, uncertainty built-in
- **vs R**: Spreadsheet integration, uncertainty quantification, easier learning curve

**Market Opportunity**: Fill the gap between powerful but complex tools (Python/R) and expensive/complex commercial software (Igor Pro), targeting scientists who need reliable uncertainty estimates without becoming programmers. 🚀

---

## Purpose

Provide comprehensive statistical analysis capabilities for experimental data in the spreadsheet, including descriptive statistics, correlation analysis, hypothesis testing, and power analysis with optional uncertainty quantification and data visualization.

---

## Features

### Descriptive Statistics (with optional uncertainties with optional confidence intervals)
- **Central Tendency**: Mean, median, mode, geometric mean, harmonic mean
- **Dispersion**: Variance, standard deviation, range, interquartile range, mean absolute deviation
- **Shape**: Skewness, kurtosis, percentiles, quartiles
- **Robust Statistics**: Trimmed mean, Winsorized mean, median absolute deviation
- **Uncertainty Quantification**: Bootstrap confidence intervals, standard errors, coefficient of variation

### Correlation Analysis (with optional uncertainties with optional confidence intervals)
- **Parametric**: Pearson correlation coefficient
- **Non-parametric**: Spearman rank correlation, Kendall tau
- **Partial Correlation**: Controlling for confounding variables
- **Confidence Intervals**: Bootstrap or Fisher z-transformation
- **Correlation Matrix**: For multivariate datasets

### Hypothesis Testing
- **t-Tests**: One-sample, two-sample (paired/unpaired), Welch's unequal variances ✅ IMPLEMENTED
- **ANOVA**: One-way, two-way, repeated measures, MANOVA ✅ PARTIALLY IMPLEMENTED (one-way ANOVA)
- **Chi-Square Tests**: Goodness-of-fit, test of independence, McNemar's test ✅ IMPLEMENTED
- **Proportion Tests**: One-sample, two-sample z-tests for proportions
- **Non-Parametric Tests**: Mann-Whitney U, Wilcoxon signed-rank, Kruskal-Wallis, Friedman test

### Power Analysis
- **Sample Size Calculation**: For means, proportions, correlations, ANOVA effects ✅ IMPLEMENTED
- **Power Curves**: Visualization of power vs. sample size/effect size ✅ IMPLEMENTED
- **Post-hoc Power**: Observed power for completed studies ✅ IMPLEMENTED
- **Sensitivity Analysis**: Minimum detectable effect sizes

### Data Visualization
- **Histograms**: Distribution visualization with kernel density estimation (reuse existing HistogramChart component)
- **Q-Q Plots**: Normality assessment using scatter plots
- **Box Plots**: Comparative distributions using ECharts boxplot series
- **Scatter Plots**: Correlation visualization with confidence ellipses
- **Power Curves**: Interactive power analysis plots using line charts

### Advanced Features
- **Normality Tests**: Shapiro-Wilk, Kolmogorov-Smirnov, Anderson-Darling
- **Confidence Intervals**: Bootstrap percentile and BCa methods, asymptotic intervals, exact intervals
- **Effect Sizes**: Cohen's d, Hedges' g, eta-squared, omega-squared, odds ratios, relative risk
- **Multiple Testing Corrections**: Bonferroni, Holm-Bonferroni, Benjamini-Hochberg FDR, q-value methods
- **Bayesian Statistics**: Credible intervals using conjugate priors, Bayes factors, posterior distributions
- **Goodness-of-Fit Tests**: Kolmogorov-Smirnov, Anderson-Darling, chi-square for continuous distributions
- **Homogeneity Tests**: Levene's test, Bartlett's test for equal variances
- **Robust Statistics**: Trimmed means, Winsorized statistics, M-estimators, resistant regression methods
- **Uncertainty Propagation**: Analytical error propagation as alternative to bootstrap for simple statistics

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Statistical Analysis            [X] │
├─────────────────────────────────────┤
│ Data Selection:                     │
│  Input Range:  [A1:A100] [Select]  │
│  Group Ranges: [B1:B50] [C1:C50]   │
│                                     │
│ Analysis Type: [Descriptive ▼]     │
│  (•) Descriptive Statistics        │
│  ( ) Correlation Analysis          │
│  ( ) Hypothesis Testing            │
│  ( ) Power Analysis                │
│                                     │
│ ┌───────────────────────────────┐ │
│ │ Method-Specific Parameters    │ │
│ │                               │ │
│ │ [✓] Include uncertainties     │ │
│ │ Confidence Level: 95%         │ │
│ │ Bootstrap Samples: 1000       │ │
│ │                               │ │
│ │ [Histogram] [Q-Q Plot]        │ │
│ └───────────────────────────────┘ │
│                                     │
│ Results:                            │
│ ┌─────────────────────────────┐   │
│ │ Mean:        45.23 ± 2.34   │   │
│ │ Std Dev:     12.45 ± 1.23   │   │
│ │ Skewness:    0.123 ± 0.045  │   │
│ │ Kurtosis:   -0.234 ± 0.089  │   │
│ │                             │   │
│ │ 95% CI: [40.67, 49.79]      │   │
│ │ Bootstrap CI: [41.12, 49.34]│   │
│ └─────────────────────────────┘   │
│                                     │
│ Visualization:                      │
│ ┌─────────────────────────────┐   │
│ │        ▄▄▄▄▄▄▄▄▄▄▄▄▄        │   │
│ │      ▄▄██████████████▄▄      │   │
│ │    ▄████████████████████▄    │   │
│ │  ▄████████████████████████▄  │   │
│ │ ▄██████████████████████████▄ │   │
│ │█████████████████████████████│   │
│ │█████████████████████████████│   │
│ │█████████████████████████████│   │
│ │ ▀██████████████████████████▀ │   │
│ │  ▀████████████████████████▀  │   │
│ │    ▀████████████████████▀    │   │
│ │      ▀▀██████████████▀▀      │   │
│ │        ▀▀▀▀▀▀▀▀▀▀▀▀▀        │   │
│ │                             │   │
│ │ Normal Distribution Fit     │   │
│ └─────────────────────────────┘   │
│                                     │
│ [Export Results] [Save to Sheet]   │
└─────────────────────────────────────┘
```

### Hypothesis Testing Panel
```
┌───────────────────────────────┐
│ Hypothesis Testing            │
│                               │
│ Test Type: [t-Test ▼]         │
│  ( ) t-Test                   │
│  ( ) ANOVA                    │
│  ( ) Chi-Square               │
│  ( ) Proportion Test          │
│  ( ) Non-Parametric           │
│                               │
│ Parameters:                   │
│  α (alpha): 0.05              │
│  Alternative: Two-sided      │
│                               │
│ Results:                      │
│  t-statistic: 2.345           │
│  df: 98                       │
│  p-value: 0.021               │
│  Cohen's d: 0.473             │
│                               │
│  Power (1-β): 0.812           │
│  95% CI: [0.123, 1.234]      │
│                               │
│ [✓] Bonferroni correction     │
│ [ ] Holm-Bonferroni           │
│ [ ] FDR (Benjamini-Hochberg)  │
└───────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Read → Analyze → Visualize → Export (Pattern B)

1. User selects data ranges from spreadsheet
2. Sidebar reads values from Univer spreadsheet
3. User selects analysis type and configures parameters
4. Apply statistical algorithms (in Rust backend)
5. Display results with uncertainties and visualizations
6. User can export results to new sheet locations
7. Optional: Save analysis metadata and parameters

---

## Technical Implementation

### TypeScript Interfaces

```typescript
interface StatisticalAnalysisSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef;
  onSelectionChange: (cellRef: string) => void;
}

type AnalysisType = 
  | 'descriptive'
  | 'correlation' 
  | 'hypothesis_testing'
  | 'power_analysis';

type HypothesisTest = 
  | 't_test_one_sample'
  | 't_test_two_sample'
  | 't_test_paired'
  | 'anova_one_way'
  | 'anova_two_way'
  | 'chi_square_goodness'
  | 'chi_square_independence'
  | 'proportion_test'
  | 'mann_whitney'
  | 'wilcoxon_signed_rank'
  | 'kruskal_wallis';

interface AnalysisConfig {
  type: AnalysisType;
  testType?: HypothesisTest;
  alpha: number; // Significance level
  confidenceLevel: number; // For CIs
  includeUncertainties: boolean;
  bootstrapSamples?: number;
  multipleTestingCorrection?: 'none' | 'bonferroni' | 'holm' | 'fdr';
  effectSize?: boolean;
}

interface DescriptiveStats {
  count: number;
  mean: number;
  median: number;
  mode?: number[];
  variance: number;
  stdDev: number;
  skewness: number;
  kurtosis: number;
  min: number;
  max: number;
  range: number;
  q1: number;
  q3: number;
  iqr: number;
  // With uncertainties
  mean_se?: number;
  mean_ci_lower?: number;
  mean_ci_upper?: number;
  skewness_se?: number;
  kurtosis_se?: number;
}

interface CorrelationResult {
  coefficient: number;
  pValue: number;
  confidenceInterval?: [number, number];
  method: 'pearson' | 'spearman' | 'kendall';
}

interface HypothesisTestResult {
  statistic: number;
  pValue: number;
  degreesOfFreedom?: number;
  effectSize?: number;
  power?: number;
  confidenceInterval?: [number, number];
  testType: HypothesisTest;
}

interface PowerAnalysisResult {
  requiredSampleSize?: number;
  achievedPower: number;
  effectSize: number;
  alpha: number;
  beta: number;
}

interface StatisticalAnalysisState {
  inputRanges: string[];
  config: AnalysisConfig;
  data: number[][];
  results: DescriptiveStats | CorrelationResult | HypothesisTestResult | PowerAnalysisResult | null;
  visualizationData: any;
  isProcessing: boolean;
  error: string | null;
}
```

---

## Algorithm Descriptions

### Descriptive Statistics

**Mean**: The arithmetic mean is calculated as:
\[\bar{x} = \frac{1}{n} \sum_{i=1}^n x_i\]

**Standard Deviation**: Population standard deviation (Bessel's correction for sample):
\[s = \sqrt{\frac{1}{n-1} \sum_{i=1}^n (x_i - \bar{x})^2}\]

**Skewness**: Measure of asymmetry:
\[g_1 = \frac{\frac{1}{n} \sum_{i=1}^n (x_i - \bar{x})^3}{s^3}\]

**Kurtosis**: Measure of tail heaviness:
\[g_2 = \frac{\frac{1}{n} \sum_{i=1}^n (x_i - \bar{x})^4}{s^4} - 3\]

**Bootstrap Confidence Intervals**: For robust uncertainty quantification:
1. Resample dataset with replacement n times
2. Calculate statistic for each resample
3. Take percentiles of bootstrap distribution (e.g., 2.5th and 97.5th for 95% CI)

### Correlation Analysis

**Pearson Correlation Coefficient**:
\[r = \frac{\sum_{i=1}^n (x_i - \bar{x})(y_i - \bar{y})}{\sqrt{\sum_{i=1}^n (x_i - \bar{x})^2} \sqrt{\sum_{i=1}^n (y_i - \bar{y})^2}}\]

**Spearman Rank Correlation**: Monotonic relationship measure using ranks.

**Confidence Intervals**: Using Fisher z-transformation:
\[z = \frac{1}{2} \ln\left(\frac{1+r}{1-r}\right)\]
\[SE_z = \frac{1}{\sqrt{n-3}}\]
\[CI = \tanh(z \pm z_{\alpha/2} \cdot SE_z)\]

### Hypothesis Testing

**t-Test**: For two independent samples:
\[t = \frac{\bar{x}_1 - \bar{x}_2}{\sqrt{\frac{s_1^2}{n_1} + \frac{s_2^2}{n_2}}}\]

**ANOVA F-Statistic**: For one-way ANOVA:
\[F = \frac{\sum_{j=1}^k n_j (\bar{y}_j - \bar{y})^2 / (k-1)}{\sum_{j=1}^k \sum_{i=1}^{n_j} (y_{ij} - \bar{y}_j)^2 / (N-k)}\]

**Chi-Square Test**: For independence:
\[\chi^2 = \sum_{i=1}^r \sum_{j=1}^c \frac{(O_{ij} - E_{ij})^2}{E_{ij}}\]
where \(E_{ij} = \frac{row_i \cdot column_j}{total}\)

**Non-Parametric Tests**: 
- Mann-Whitney U: Ranks-based comparison
- Kruskal-Wallis: Extension of Mann-Whitney to k groups

### Power Analysis

**Power for t-Test**:
\[\beta = \Phi\left(z_{\alpha/2} - \frac{\delta}{\sigma} \sqrt{\frac{n}{4}}\right) + \Phi\left(-z_{\alpha/2} - \frac{\delta}{\sigma} \sqrt{\frac{n}{4}}\right)\]
where \(\delta\) is effect size, \(\sigma\) is standard deviation.

**Sample Size Calculation**: Solve for n in power equations using numerical methods.

### Proportion Tests

**z-Test for Proportions**:
\[z = \frac{\hat{p}_1 - \hat{p}_2}{\sqrt{\hat{p}(1-\hat{p})(\frac{1}{n_1} + \frac{1}{n_2})}}\]
where \(\hat{p} = \frac{n_1\hat{p}_1 + n_2\hat{p}_2}{n_1 + n_2}\)

### Effect Sizes

**Cohen's d**: Standardized mean difference
\[d = \frac{\bar{x}_1 - \bar{x}_2}{s_{pooled}}\]

**Eta-squared**: Proportion of variance explained in ANOVA
\[\eta^2 = \frac{SS_{between}}{SS_{total}}\]

### Multiple Testing Correction

**Bonferroni**: \(\alpha_{corrected} = \frac{\alpha}{m}\)

**Benjamini-Hochberg FDR**: Rank p-values, compare to \(\frac{i}{m} \cdot \alpha\)

---

## Rust Backend Implementation

**IMPORTANT**: ALL statistical algorithms are implemented in Rust. TypeScript ONLY calls Rust commands via invoke().

### TypeScript Wrapper (UI Only)

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function performStatisticalAnalysis(
  data: number[][],
  config: AnalysisConfig
): Promise<StatisticalAnalysisResult> {
  return await invoke('perform_statistical_analysis', {
    data,
    config
  });
}
```

### Rust Statistical Module

```rust
// AnaFis/src-tauri/src/statistics/mod.rs

use ndarray::Array2;
use statrs::statistics::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct StatisticalRequest {
    data: Vec<Vec<f64>>,
    analysis_type: String,
    test_type: Option<String>,
    alpha: f64,
    confidence_level: f64,
    include_uncertainties: bool,
    bootstrap_samples: Option<usize>,
}

#[derive(Serialize)]
pub struct StatisticalResponse {
    results: serde_json::Value, // Flexible for different result types
    visualization_data: Option<VisualizationData>,
}

#[tauri::command]
pub async fn perform_statistical_analysis(
    request: StatisticalRequest,
) -> Result<StatisticalResponse, String> {
    let data = request.data;
    
    match request.analysis_type.as_str() {
        "descriptive" => {
            let stats = calculate_descriptive_stats(&data[0], request.include_uncertainties, request.bootstrap_samples)?;
            Ok(StatisticalResponse {
                results: serde_json::to_value(stats)?,
                visualization_data: Some(generate_histogram_data(&data[0])),
            })
        },
        "correlation" => {
            let correlation = calculate_correlation(&data[0], &data[1], request.confidence_level)?;
            Ok(StatisticalResponse {
                results: serde_json::to_value(correlation)?,
                visualization_data: Some(generate_scatter_data(&data[0], &data[1])),
            })
        },
        "hypothesis_testing" => {
            let test_result = perform_hypothesis_test(&data, &request.test_type.unwrap(), request.alpha)?;
            Ok(StatisticalResponse {
                results: serde_json::to_value(test_result)?,
                visualization_data: None,
            })
        },
        "power_analysis" => {
            let power_result = calculate_power(&data, request.alpha)?;
            Ok(StatisticalResponse {
                results: serde_json::to_value(power_result)?,
                visualization_data: Some(generate_power_curve_data()),
            })
        },
        _ => Err(format!("Unknown analysis type: {}", request.analysis_type)),
    }
}

// Descriptive statistics with bootstrap uncertainties
fn calculate_descriptive_stats(
    data: &[f64], 
    include_uncertainties: bool, 
    bootstrap_samples: Option<usize>
) -> Result<DescriptiveStats, String> {
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();
    
    // Higher moments
    let skewness = data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / n;
    let kurtosis = data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum::<f64>() / n - 3.0;
    
    let mut stats = DescriptiveStats {
        count: data.len(),
        mean,
        variance,
        std_dev,
        skewness,
        kurtosis,
        // ... other fields
    };
    
    if include_uncertainties {
        let bootstrap_results = bootstrap_statistics(data, bootstrap_samples.unwrap_or(1000));
        stats.mean_se = Some(bootstrap_results.mean_se);
        stats.mean_ci_lower = Some(bootstrap_results.mean_ci[0]);
        stats.mean_ci_upper = Some(bootstrap_results.mean_ci[1]);
        // ... similar for other statistics
    }
    
    Ok(stats)
}

// Bootstrap implementation
fn bootstrap_statistics(data: &[f64], n_samples: usize) -> BootstrapResults {
    let mut rng = rand::thread_rng();
    let mut bootstrap_means = Vec::with_capacity(n_samples);
    
    for _ in 0..n_samples {
        let sample: Vec<f64> = (0..data.len())
            .map(|_| *data.choose(&mut rng).unwrap())
            .collect();
        let mean = sample.iter().sum::<f64>() / sample.len() as f64;
        bootstrap_means.push(mean);
    }
    
    bootstrap_means.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean_se = bootstrap_means.iter().map(|x| (x - bootstrap_means.iter().sum::<f64>() / n_samples as f64).powi(2)).sum::<f64>().sqrt() / (n_samples as f64 - 1.0).sqrt();
    
    BootstrapResults {
        mean_se,
        mean_ci: [bootstrap_means[(n_samples * 5 / 100)], bootstrap_means[(n_samples * 95 / 100)]],
    }
}

// Correlation with confidence intervals
fn calculate_correlation(x: &[f64], y: &[f64], confidence_level: f64) -> Result<CorrelationResult, String> {
    let n = x.len() as f64;
    let sum_x = x.iter().sum::<f64>();
    let sum_y = y.iter().sum::<f64>();
    let sum_xy = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum::<f64>();
    let sum_x2 = x.iter().map(|a| a * a).sum::<f64>();
    let sum_y2 = y.iter().map(|a| a * a).sum::<f64>();
    
    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
    
    let r = numerator / denominator;
    
    // Fisher z-transformation for CI
    let z = 0.5 * ((1.0 + r) / (1.0 - r)).ln();
    let se_z = 1.0 / (n - 3.0).sqrt();
    let z_crit = statrs::distribution::Normal::new(0.0, 1.0).unwrap()
        .inverse_cdf(1.0 - (1.0 - confidence_level) / 2.0);
    
    let ci_lower = ((z - z_crit * se_z).exp() - 1.0) / ((z - z_crit * se_z).exp() + 1.0);
    let ci_upper = ((z + z_crit * se_z).exp() - 1.0) / ((z + z_crit * se_z).exp() + 1.0);
    
    Ok(CorrelationResult {
        coefficient: r,
        confidence_interval: Some([ci_lower, ci_upper]),
        // ... other fields
    })
}

// Hypothesis testing implementations
fn perform_hypothesis_test(data: &[Vec<f64>], test_type: &str, alpha: f64) -> Result<HypothesisTestResult, String> {
    match test_type {
        "t_test_two_sample" => {
            let group1 = &data[0];
            let group2 = &data[1];
            
            let mean1 = group1.iter().sum::<f64>() / group1.len() as f64;
            let mean2 = group2.iter().sum::<f64>() / group2.len() as f64;
            
            let var1 = group1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (group1.len() - 1) as f64;
            let var2 = group2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (group2.len() - 1) as f64;
            
            let se = (var1 / group1.len() as f64 + var2 / group2.len() as f64).sqrt();
            let t = (mean1 - mean2) / se;
            
            let df = ((var1 / group1.len() as f64 + var2 / group2.len() as f64).powi(2)) / 
                    ((var1 / group1.len() as f64).powi(2) / (group1.len() - 1) as f64 + 
                     (var2 / group2.len() as f64).powi(2) / (group2.len() - 1) as f64);
            
            let p_value = 2.0 * (1.0 - statrs::distribution::StudentsT::new(0.0, 1.0, df).unwrap().cdf(t.abs()));
            
            // Cohen's d
            let pooled_sd = ((var1 * (group1.len() - 1) as f64 + var2 * (group2.len() - 1) as f64) / 
                           (group1.len() + group2.len() - 2) as f64).sqrt();
            let cohens_d = (mean1 - mean2) / pooled_sd;
            
            Ok(HypothesisTestResult {
                statistic: t,
                p_value,
                degrees_of_freedom: Some(df),
                effect_size: Some(cohens_d),
                // ... other fields
            })
        },
        // ... implementations for other tests
        _ => Err(format!("Test type {} not implemented", test_type)),
    }
}
```

---

## Suggested Separate Components

To maintain focus and manage complexity, the following advanced features are recommended as separate sidebars or tabs:

### Regression Analysis Sidebar
- Simple and multiple linear regression
- Logistic regression
- Polynomial regression
- Diagnostics (residual plots, influence measures, multicollinearity checks)
- Cross-validation (K-fold CV, leave-one-out CV)

### Time Series Analysis Tab
- Autocorrelation functions
- Partial autocorrelation
- ARIMA modeling basics
- Stationarity tests (Dickey-Fuller)

### Quality Control Sidebar
- Control charts (X-bar, R, p, c charts)
- Process capability indices (Cp, Cpk)
- Six Sigma metrics

### Reliability Analysis Sidebar
- Mean time between failures (MTBF)
- Failure rate calculations
- Weibull analysis
- Reliability growth modeling

### Multivariate Analysis Sidebar
- Principal component analysis (PCA)
- Factor analysis
- Discriminant analysis
- Canonical correlation
- Dimensionality reduction (MDS, t-SNE)

### Clustering Sidebar
- K-means clustering
- Hierarchical clustering
- Gaussian mixture models
- Validation metrics (silhouette scores)

### Survival Analysis Sidebar
- Kaplan-Meier estimators
- Log-rank tests
- Cox proportional hazards models

### Meta-Analysis Sidebar
- Fixed and random effects models
- Forest plots
- Heterogeneity tests (I², Q-statistic)

### Design of Experiments Sidebar
- Basic factorial designs
- Response surface methodology
- Optimal design principles

### Advanced Power Analysis Sidebar
- Power calculations for complex designs
- Repeated measures
- Clustered data
- Longitudinal studies

---

## Implementation Enhancements from Review

### Static Analysis and Code Quality
To ensure high reliability and catch bugs early:
- **Clippy**: Integrated into CI for idiomatic Rust code and common mistake detection
- **Miri**: Used for checking unsafe code blocks in bootstrap and sampling algorithms
- **Rudra**: Employed to detect potential memory safety issues in unsafe code

### Uncertainty Quantification Enhancements
- **Analytical Error Propagation**: For simple derived quantities, provide fast approximate uncertainties using:
  \[\sigma_f^2 \approx \left(\frac{\partial f}{\partial x}\right)^2 \sigma_x^2 + \left(\frac{\partial f}{\partial y}\right)^2 \sigma_y^2 + 2 \frac{\partial f}{\partial x} \frac{\partial f}{\partial y} \sigma_{xy}\]
- **Bootstrap as Default**: Use for complex statistics where analytical methods are intractable

### Hypothesis Testing Improvements
- **P-Value Precision**: Report p-values < 1e-15 as "< 1e-15" for scientific honesty
- **Distribution Validation**: Emphasize normality tests to guide test selection
- **Effect Size Integration**: Always include appropriate effect sizes with test results

### Rust Ecosystem Evaluation
- **statrs**: Primary choice for statistical distributions and tests
- **Consider statsrust**: Review for design patterns like zero-cost abstractions
- **Future High-Precision**: Evaluate astro_nalgebra for arbitrary-precision needs
- **rand Crate**: Use Uniform distributions for bootstrap sampling

---

## Architecture Notes
- ALL statistical computations implemented in Rust for precision and performance
- TypeScript handles only UI, data marshaling, and result visualization
- Bootstrap and Monte Carlo methods for uncertainty quantification
- State-of-the-art algorithms with proper numerical stability

**Benefits**:
- High-precision floating-point calculations
- Consistent results across platforms
- Fast processing of large datasets (10,000+ points)
- Memory-efficient implementations using ndarray

---

## Dependencies

### Frontend
- **ECharts** - Already installed (v6.0.0), supports all required chart types
- MathJax for mathematical notation display

### Backend
- **statrs** - Comprehensive statistical distributions and tests
- **ndarray** - High-performance array operations
- **nalgebra** - Linear algebra for advanced statistics
- **rand** - Random number generation for bootstrap (using Uniform distributions)
- **serde** - Serialization for data exchange
- **Future consideration**: astro_nalgebra for arbitrary-precision arithmetic

### Installation

```bash
# ECharts already installed in package.json (v6.0.0)
# MathJax for mathematical notation (if not already installed)
npm install react-katex katex
```

```toml
# Rust (Cargo.toml)
[dependencies]
statrs = "0.16"
ndarray = "0.15"
nalgebra = "0.32"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/StatisticalAnalysisSidebar.tsx`
- **Backend**: `AnaFis/src-tauri/src/scientific/statistics.rs`
- **Types**: `AnaFis/src/types/statistics.ts`

---

## Success Criteria

- ✓ All statistical methods produce accurate results
- ✓ Bootstrap confidence intervals match theoretical values
- ✓ Analytical error propagation provides fast uncertainty estimates
- ✓ Hypothesis tests have correct p-values with proper precision reporting ✅ IMPLEMENTED
- ✓ Normality tests guide appropriate test selection
- ✓ Effect sizes included with all relevant test results ✅ IMPLEMENTED
- ✓ Power analysis with accurate non-central distribution approximations ✅ IMPLEMENTED
- ✓ Data visualizations update in real-time using existing ECharts infrastructure
- ✓ Performance: Process 10,000 data points in < 500ms
- ✓ Memory efficient for large datasets
- ✓ Static analysis tools integrated in CI pipeline
- ✓ Export functionality works correctly

---

## Testing Plan

### Static Analysis (CI Pipeline)
- Clippy for idiomatic Rust code
- Miri for unsafe code verification
- Rudra for memory safety analysis

### Unit Tests
- Accuracy of all statistical calculations ✅ IMPLEMENTED (hypothesis testing, power analysis)
- Bootstrap CI coverage properties
- Analytical error propagation correctness
- Hypothesis test p-value distributions and precision ✅ IMPLEMENTED
- Edge cases (small samples, extreme values) ✅ IMPLEMENTED

### Integration Tests
- End-to-end analysis workflows
- Data import/export from spreadsheet
- Visualization generation
- Parameter validation

### E2E Tests
- Complete user analysis sessions
- Result interpretation and export
- Error handling and recovery

---
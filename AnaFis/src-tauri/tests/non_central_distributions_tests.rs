//! Integration tests for non-central distributions
//!
//! These tests validate non-central t and F distributions used in
//! statistical power analysis and hypothesis testing.

use anafis_lib::scientific::statistics::comprehensive_analysis::layer4_primitives::non_central_distributions::{NonCentralT, NonCentralF};

#[test]
fn test_non_central_t_cdf() {
    let nct = NonCentralT::new(2.0, 10.0).unwrap();

    // Test basic CDF properties
    assert!(nct.cdf(0.0) >= 0.0 && nct.cdf(0.0) <= 1.0);
    assert!(nct.cdf(5.0) >= nct.cdf(0.0));
    assert!(nct.cdf(-5.0) <= nct.cdf(0.0));

    // Test with zero non-centrality (should equal central t)
    let ct = NonCentralT::new(0.0, 10.0).unwrap();
    let central_cdf = ct.cdf(1.0);
    use statrs::distribution::{ContinuousCDF, StudentsT};
    let statrs_t = StudentsT::new(0.0, 1.0, 10.0).unwrap();
    let expected = statrs_t.cdf(1.0);
    assert!((central_cdf - expected).abs() < 1e-10);
}

#[test]
fn test_non_central_f_cdf() {
    let ncf = NonCentralF::new(5.0, 3.0, 20.0).unwrap();

    // Test basic CDF properties
    assert!(ncf.cdf(0.5) >= 0.0 && ncf.cdf(0.5) <= 1.0);
    assert!(ncf.cdf(2.0) >= ncf.cdf(0.5));
    assert_eq!(ncf.cdf(0.0), 0.0);

    // Test with zero non-centrality (should equal central F)
    let cf = NonCentralF::new(0.0, 3.0, 20.0).unwrap();
    let central_cdf = cf.cdf(2.0);
    use statrs::distribution::{ContinuousCDF, FisherSnedecor};
    let statrs_f = FisherSnedecor::new(3.0, 20.0).unwrap();
    let expected = statrs_f.cdf(2.0);
    assert!((central_cdf - expected).abs() < 1e-6);
}

#[test]
fn test_power_calculations() {
    let nct = NonCentralT::new(2.0, 15.0).unwrap();
    let power = nct.power_two_sided(2.131); // t-critical for α=0.05, df=15
    assert!(power > 0.0 && power < 1.0);

    let ncf = NonCentralF::new(10.0, 2.0, 30.0).unwrap();
    let power = ncf.power(3.32); // F-critical for α=0.05, df=2,30
    assert!(power > 0.0 && power < 1.0);
}

#[test]
fn test_error_conditions() {
    assert!(NonCentralT::new(1.0, 0.0).is_err());
    assert!(NonCentralT::new(f64::INFINITY, 10.0).is_err());
    assert!(NonCentralF::new(-1.0, 2.0, 20.0).is_err());
    assert!(NonCentralF::new(1.0, 0.0, 20.0).is_err());
}
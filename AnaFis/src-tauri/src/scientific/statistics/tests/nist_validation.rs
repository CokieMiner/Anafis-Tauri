//! NIST Statistical Reference Dataset Validation
//!
//! Tests against National Institute of Standards and Technology (NIST)
//! certified reference datasets to ensure compliance with established
//! statistical standards and numerical accuracy.

use super::test_utils::*;
use crate::scientific::statistics::descriptive::StatisticalMoments;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nist_michelson_data() {
        // NIST "Michelso" dataset (Speed of Light measurements)
        // Certified reference dataset from NIST StRD
        let data = vec![
            299.85, 299.74, 299.90, 300.07, 299.93, 299.85, 299.95, 299.98, 299.98,
            299.88, 300.00, 299.98, 299.93, 299.65, 299.76, 299.81, 300.00, 300.00,
            299.96, 299.96, 299.96, 299.94, 299.96, 299.94, 299.88, 299.80, 299.85,
            299.88, 299.90, 299.84, 299.83, 299.79, 299.81, 299.88, 299.88, 299.83,
            299.80, 299.79, 299.76, 299.80, 299.88, 299.88, 299.88, 299.86, 299.72,
            299.62, 299.86, 299.97, 299.95, 299.88, 299.91, 299.85, 299.87, 299.84,
            299.84, 299.85, 299.84, 299.84, 299.84, 299.89, 299.81, 299.81, 299.82,
            299.80, 299.77, 299.76, 299.74, 299.75, 299.76, 299.91, 299.92, 299.89,
            299.86, 299.88, 299.72, 299.84, 299.85, 299.85, 299.78, 299.89, 299.84,
            299.78, 299.81, 299.76, 299.81, 299.79, 299.81, 299.82, 299.85, 299.87,
            299.87, 299.81, 299.74, 299.81, 299.94, 299.95, 299.80, 299.81, 299.87, 299.72
        ];

        let mean = data.mean();
        let std = data.std_dev();
        let variance = data.variance();

        // NIST certified values for physics-grade precision
        let certified_mean = 299.852400000000;
        let certified_std = 0.0790105478190518;
        let certified_variance = certified_std * certified_std; // 0.006243

        // Strict comparison against NIST certified values
        assert!(approx_eq(mean, certified_mean, 1e-10), "Mean mismatch: computed={}, certified={}", mean, certified_mean);
        assert!(approx_eq(std, certified_std, 1e-10), "Std dev mismatch: computed={}, certified={}", std, certified_std);
        assert!(approx_eq(variance, certified_variance, 1e-10), "Variance mismatch: computed={}, certified={}", variance, certified_variance);
    }

    #[test]
    fn test_nist_uniform_data() {
        // NIST Uniform dataset for testing quantile calculations
        // This is a simplified test - in practice you'd use full NIST datasets
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        let mean = data.mean();
        let variance = data.variance();

        // For uniform data 1..10, mean should be 5.5
        assert!(approx_eq(mean, 5.5, 1e-10), "Uniform mean failed: got {}, expected 5.5", mean);

        // Variance calculation should be numerically stable
        assert!(variance > 0.0, "Variance should be positive for non-constant data");
    }

    #[test]
    fn test_nist_lew_data() {
        // NIST "Lew" dataset (Beam Deflection Data)
        // Certified reference dataset from NIST StRD
        let data: Vec<f64> = vec![
            -213.0, -564.0, -35.0, -15.0, 141.0, 115.0, -420.0, -360.0, 203.0, -338.0, -431.0, 194.0, -220.0, -513.0, 154.0,
            -125.0, -559.0, 92.0, -21.0, -579.0, -52.0, 99.0, -543.0, -175.0, 162.0, -457.0, -346.0, 204.0, -300.0, -474.0,
            164.0, -107.0, -572.0, -8.0, 83.0, -541.0, -224.0, 180.0, -420.0, -374.0, 201.0, -236.0, -531.0, 83.0, 27.0,
            -564.0, -112.0, 131.0, -507.0, -254.0, 199.0, -311.0, -495.0, 143.0, -46.0, -579.0, -90.0, 136.0, -472.0, -338.0,
            202.0, -287.0, -477.0, 169.0, -124.0, -568.0, 17.0, 48.0, -568.0, -135.0, 162.0, -430.0, -422.0, 172.0, -74.0,
            -577.0, -13.0, 92.0, -534.0, -243.0, 194.0, -355.0, -465.0, 156.0, -81.0, -578.0, -64.0, 139.0, -449.0, -384.0,
            193.0, -198.0, -538.0, 110.0, -44.0, -577.0, -6.0, 66.0, -552.0, -164.0, 161.0, -460.0, -344.0, 205.0, -281.0,
            -504.0, 134.0, -28.0, -576.0, -118.0, 156.0, -437.0, -381.0, 200.0, -220.0, -540.0, 83.0, 11.0, -568.0, -160.0,
            172.0, -414.0, -408.0, 188.0, -125.0, -572.0, -32.0, 139.0, -492.0, -321.0, 205.0, -262.0, -504.0, 142.0, -83.0,
            -574.0, 0.0, 48.0, -571.0, -106.0, 137.0, -501.0, -266.0, 190.0, -391.0, -406.0, 194.0, -186.0, -553.0, 83.0, -13.0,
            -577.0, -49.0, 103.0, -515.0, -280.0, 201.0, 300.0, -506.0, 131.0, -45.0, -578.0, -80.0, 138.0, -462.0, -361.0,
            201.0, -211.0, -554.0, 32.0, 74.0, -533.0, -235.0, 187.0, -372.0, -442.0, 182.0, -147.0, -566.0, 25.0, 68.0,
            -535.0, -244.0, 194.0, -351.0, -463.0, 174.0, -125.0, -570.0, 15.0, 72.0, -550.0, -190.0, 172.0, -424.0, -385.0,
            198.0, -218.0, -536.0, 96.0
        ];

        let mean = data.mean();
        let std = data.std_dev();
        let variance = data.variance();

        // NIST certified values for physics-grade precision
        let certified_mean = -177.435000000000;
        let certified_std = 277.332168044316;
        let certified_variance = certified_std * certified_std; // approximately 76900

        // Strict comparison against NIST certified values
        assert!(approx_eq(mean, certified_mean, 1e-10), "Mean mismatch: computed={}, certified={}", mean, certified_mean);
        assert!(approx_eq(std, certified_std, 1e-10), "Std dev mismatch: computed={}, certified={}", std, certified_std);
        assert!(approx_eq(variance, certified_variance, 1e-10), "Variance mismatch: computed={}, certified={}", variance, certified_variance);
    }
}
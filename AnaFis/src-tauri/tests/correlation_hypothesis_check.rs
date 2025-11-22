
#[cfg(test)]
mod tests {
    use anafis_lib::scientific::statistics::correlation::CorrelationMethods;
    use anafis_lib::scientific::statistics::hypothesis_testing::HypothesisTestingEngine;

    #[test]
    fn test_autocorrelation_standard_definition() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // Mean = 3.0
        // Variance (sample) = 2.5
        // Denominator = 2.5 * 4 = 10.0
        
        // Lag 1:
        // (1-3)(2-3) + (2-3)(3-3) + (3-3)(4-3) + (4-3)(5-3)
        // (-2)(-1) + (-1)(0) + (0)(1) + (1)(2)
        // 2 + 0 + 0 + 2 = 4.0
        // r_1 = 4.0 / 10.0 = 0.4
        
        let acf = CorrelationMethods::autocorrelation(&data, 1).unwrap();
        assert!((acf[0] - 0.4).abs() < 1e-10, "Lag 1 autocorrelation incorrect: expected 0.4, got {}", acf[0]);
    }

    #[test]
    fn test_tukey_hsd_p_value_nan() {
        let group1 = vec![10.0, 12.0, 11.0];
        let group2 = vec![15.0, 16.0, 14.0]; // Significantly different
        let group3 = vec![10.5, 11.5, 12.5]; // Similar to group1
        
        let groups = vec![group1.as_slice(), group2.as_slice(), group3.as_slice()];
        
        // We need to run ANOVA first to get MS_within
        let anova = HypothesisTestingEngine::one_way_anova(&groups).unwrap();
        let post_hoc = anova.post_hoc_results.unwrap();
        
        assert!(!post_hoc.is_empty());
        
        // Check significance (Group 1 vs Group 2 should be significant)
        // Group 1 mean ~ 11, Group 2 mean ~ 15. Difference 4.
        // MS_within should be small.
        let g1_vs_g2 = post_hoc.iter().find(|r| r.comparison.contains("Group 1 vs Group 2")).unwrap();
        assert!(g1_vs_g2.significant, "Group 1 vs Group 2 should be significant");
    }

    #[test]
    fn test_two_way_anova_unbalanced_success() {
        // Test data with unbalanced design (different sample sizes per cell)
        let data = vec![
            vec![1.0, 2.0], // Cell 0,0 (n=2)
            vec![3.0],      // Cell 0,1 (n=1) - Unbalanced!
            vec![4.0, 5.0], // Cell 1,0 (n=2)
            vec![6.0, 7.0], // Cell 1,1 (n=2)
        ];
        let factor1 = vec![0, 0, 1, 1];
        let factor2 = vec![0, 1, 0, 1];

        let result = HypothesisTestingEngine::two_way_anova(&data, &factor1, &factor2);
        assert!(result.is_ok(), "Two-way ANOVA should now handle unbalanced designs");

        let anova_result = result.unwrap();

        // Verify that we get reasonable results
        assert!(anova_result.f_statistic_factor1 >= 0.0);
        assert!(anova_result.f_statistic_factor2 >= 0.0);
        assert!(anova_result.f_statistic_interaction >= 0.0);
        assert!(anova_result.p_value_factor1 >= 0.0 && anova_result.p_value_factor1 <= 1.0);
        assert!(anova_result.p_value_factor2 >= 0.0 && anova_result.p_value_factor2 <= 1.0);
        assert!(anova_result.p_value_interaction >= 0.0 && anova_result.p_value_interaction <= 1.0);

        // Check degrees of freedom
        assert_eq!(anova_result.degrees_of_freedom_factor1, 1.0); // 2 levels - 1
        assert_eq!(anova_result.degrees_of_freedom_factor2, 1.0); // 2 levels - 1
        assert_eq!(anova_result.degrees_of_freedom_interaction, 1.0); // 1 * 1
        assert_eq!(anova_result.degrees_of_freedom_residual, 3.0); // 7 observations - 4 parameters (full model)
    }

    #[test]
    fn test_two_way_anova_balanced_design() {
        // Test data with balanced design (equal sample sizes per cell)
        let data = vec![
            vec![1.0, 2.0], // Cell 0,0 (n=2)
            vec![3.0, 4.0], // Cell 0,1 (n=2) - Balanced!
            vec![5.0, 6.0], // Cell 1,0 (n=2)
            vec![7.0, 8.0], // Cell 1,1 (n=2)
        ];
        let factor1 = vec![0, 0, 1, 1];
        let factor2 = vec![0, 1, 0, 1];

        let result = HypothesisTestingEngine::two_way_anova(&data, &factor1, &factor2);
        assert!(result.is_ok(), "Two-way ANOVA should handle balanced designs");

        let anova_result = result.unwrap();

        // Verify that we get reasonable results
        assert!(anova_result.f_statistic_factor1 >= 0.0);
        assert!(anova_result.f_statistic_factor2 >= 0.0);
        assert!(anova_result.f_statistic_interaction >= 0.0);
        assert!(anova_result.p_value_factor1 >= 0.0 && anova_result.p_value_factor1 <= 1.0);
        assert!(anova_result.p_value_factor2 >= 0.0 && anova_result.p_value_factor2 <= 1.0);
        assert!(anova_result.p_value_interaction >= 0.0 && anova_result.p_value_interaction <= 1.0);

        // Check degrees of freedom
        assert_eq!(anova_result.degrees_of_freedom_factor1, 1.0); // 2 levels - 1
        assert_eq!(anova_result.degrees_of_freedom_factor2, 1.0); // 2 levels - 1
        assert_eq!(anova_result.degrees_of_freedom_interaction, 1.0); // 1 * 1
        assert_eq!(anova_result.degrees_of_freedom_residual, 4.0); // 8 observations - 4 parameters
    }
}

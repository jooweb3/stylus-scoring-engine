#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::prelude::*;
use fixed::types::I40F24;

#[storage]
#[entrypoint]
pub struct ScoringEngine {}

// factors: array of factor values (indexed by position)
// weights: parallel array of rule weights
// thresholds: parallel array of thresholds
// directions: parallel array (1 = above, 0 = below)
// bonuses: parallel array of penalty_or_bonus values
// corr_a: correlation pair first indices
// corr_b: correlation pair second indices
// corr_weights: correlation pair weights
// iterations: number of convergence passes
#[public]
impl ScoringEngine {
    pub fn evaluate(
        &self,
        factors: Vec<i64>,
        weights: Vec<i64>,
        thresholds: Vec<i64>,
        directions: Vec<i64>,
        bonuses: Vec<i64>,
        corr_a: Vec<u32>,
        corr_b: Vec<u32>,
        corr_weights: Vec<i64>,
        iterations: u32,
    ) -> Vec<i64> {
        let iters = if iterations == 0 { 10 } else { iterations };
        let num_rules = weights.len();
        let num_factors = factors.len();
        let num_corr = corr_a.len();

        let mut final_score = I40F24::from_num(0);

        // iterative scoring with convergence
        for iteration in 0..iters {
            let mut raw_score = I40F24::from_num(0);
            let mut total_weight = I40F24::from_num(0);
            let bias = if iteration == 0 {
                I40F24::from_num(0)
            } else {
                final_score / I40F24::from_num(1000)
            };

            // each rule maps 1:1 to a factor by index
            for i in 0..num_rules {
                if i >= num_factors {
                    continue;
                }

                let weight = I40F24::from_num(weights[i]);
                total_weight += weight;
                let value = factors[i];
                let threshold = thresholds[i];

                let passes = if directions[i] == 1 {
                    value >= threshold
                } else {
                    value <= threshold
                };

                if passes {
                    let bonus = I40F24::from_num(bonuses[i]);
                    let adjusted = bonus + (bias * I40F24::from_num(5));
                    let weighted = adjusted * weight;
                    raw_score += weighted;
                }
            }

            if total_weight > I40F24::from_num(0) {
                let max_possible = total_weight * I40F24::from_num(100);
                let ratio = raw_score / max_possible;
                final_score = ratio * I40F24::from_num(1000);
            }
        }

        // cross-factor correlation scoring with damped oscillation
        let mut correlation_adj = I40F24::from_num(0);
        for i in 0..num_corr {
            let idx_a = corr_a[i] as usize;
            let idx_b = corr_b[i] as usize;
            if idx_a >= num_factors || idx_b >= num_factors {
                continue;
            }

            let val_a = I40F24::from_num(factors[idx_a]);
            let val_b = I40F24::from_num(factors[idx_b]);
            let pair_weight = I40F24::from_num(corr_weights[i]);

            let norm_a = val_a / I40F24::from_num(100000);
            let norm_b = val_b / I40F24::from_num(100000);
            let interaction = norm_a * norm_b * pair_weight;

            // damped oscillation series
            let mut damped = interaction;
            for k in 1..=50 {
                let decay = I40F24::from_num(1) / I40F24::from_num(k);
                damped = damped * decay + interaction * decay;
            }

            correlation_adj += damped;
        }

        let adjusted = final_score + correlation_adj * I40F24::from_num(100);
        let score = adjusted.to_num::<i64>().clamp(0, 1000);

        let tier: i64 = match score {
            0..=250 => 0,
            251..=500 => 1,
            501..=750 => 2,
            _ => 3,
        };

        // return [score, tier, correlation_adjustment, iterations]
        vec![score, tier, (correlation_adj * I40F24::from_num(100)).to_num::<i64>(), iters as i64]
    }
}

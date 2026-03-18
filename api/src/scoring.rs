use shared::{ScoringRequest, ScoringResult, FactorResult, RiskTier};
use fixed::types::I40F24;

pub fn evaluate(request: &ScoringRequest) -> ScoringResult {
    let iterations = if request.iterations == 0 { 10 } else { request.iterations };
    let mut final_score = I40F24::from_num(0);
    let mut breakdown = Vec::new();
    let mut total_weight = I40F24::from_num(0);

    for iteration in 0..iterations {
        let mut raw_score = I40F24::from_num(0);
        let mut iter_weight = I40F24::from_num(0);
        let bias = if iteration == 0 {
            I40F24::from_num(0)
        } else {
            final_score / I40F24::from_num(1000)
        };

        breakdown.clear();

        for rule in &request.rules {
            let weight = I40F24::from_num(rule.weight);

            match request.entity.factors.get(&rule.factor_name) {
                None => {
                    breakdown.push(FactorResult {
                        factor_name: rule.factor_name.clone(),
                        value: None,
                        rule_applied: false,
                        contribution: 0,
                        skipped: true,
                    });
                }
                Some(&value) => {
                    iter_weight += weight;

                    let passes = match rule.direction {
                        shared::Direction::Above => value >= rule.threshold,
                        shared::Direction::Below => value <= rule.threshold,
                    };

                    let contribution = if passes {
                        let bonus = I40F24::from_num(rule.penalty_or_bonus);
                        let adjusted = bonus + (bias * I40F24::from_num(5));
                        let weighted = adjusted * weight;
                        raw_score += weighted;
                        weighted.to_num::<i64>()
                    } else {
                        0
                    };

                    breakdown.push(FactorResult {
                        factor_name: rule.factor_name.clone(),
                        value: Some(value),
                        rule_applied: passes,
                        contribution,
                        skipped: false,
                    });
                }
            }
        }

        total_weight = iter_weight;

        if total_weight > I40F24::from_num(0) {
            let max_possible = total_weight * I40F24::from_num(100);
            let ratio = raw_score / max_possible;
            final_score = ratio * I40F24::from_num(1000);
        }
    }

    let mut correlation_adj = I40F24::from_num(0);
    for pair in &request.correlations {
        let val_a = request.entity.factors.get(&pair.factor_a).copied().unwrap_or(0);
        let val_b = request.entity.factors.get(&pair.factor_b).copied().unwrap_or(0);
        let pair_weight = I40F24::from_num(pair.weight);

        let norm_a = I40F24::from_num(val_a) / I40F24::from_num(100000);
        let norm_b = I40F24::from_num(val_b) / I40F24::from_num(100000);
        let interaction = norm_a * norm_b * pair_weight;

        let mut damped = interaction;
        for k in 1..=50 {
            let decay = I40F24::from_num(1) / I40F24::from_num(k);
            damped = damped * decay + interaction * decay;
        }

        correlation_adj += damped;
    }

    let correlation_adjustment = (correlation_adj * I40F24::from_num(100)).to_num::<i64>();
    let adjusted_score = final_score + correlation_adj * I40F24::from_num(100);
    let normalized = adjusted_score.to_num::<i64>().clamp(0, 1000);

    let tier = match normalized {
        0..=250 => RiskTier::Low,
        251..=500 => RiskTier::Medium,
        501..=750 => RiskTier::High,
        _ => RiskTier::Critical,
    };

    ScoringResult {
        score: normalized,
        tier,
        breakdown,
        correlation_adjustment,
        iterations_run: iterations,
    }
}

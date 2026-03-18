#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub factors: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    Above,
    Below,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub factor_name: String,
    pub weight: i64,
    pub threshold: i64,
    pub direction: Direction,
    pub penalty_or_bonus: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationPair {
    pub factor_a: String,
    pub factor_b: String,
    pub weight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringRequest {
    pub entity: Entity,
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub correlations: Vec<CorrelationPair>,
    #[serde(default = "default_iterations")]
    pub iterations: u32,
}

fn default_iterations() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTier {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorResult {
    pub factor_name: String,
    pub value: Option<i64>,
    pub rule_applied: bool,
    pub contribution: i64,
    pub skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
    pub score: i64,
    pub tier: RiskTier,
    pub breakdown: Vec<FactorResult>,
    pub correlation_adjustment: i64,
    pub iterations_run: u32,
}

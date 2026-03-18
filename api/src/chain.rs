use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;
use alloy::sol;
use shared::ScoringRequest;
use std::str::FromStr;

sol! {
    #[sol(rpc)]
    interface IScoringEngine {
        function evaluate(
            int64[] memory factors,
            int64[] memory weights,
            int64[] memory thresholds,
            int64[] memory directions,
            int64[] memory bonuses,
            uint32[] memory corrA,
            uint32[] memory corrB,
            int64[] memory corrWeights,
            uint32 iterations
        ) external view returns (int64[] memory);
    }
}

sol! {
    #[sol(rpc)]
    interface IScoringEngineSol {
        function evaluate(
            int64[] calldata factors,
            int64[] calldata weights,
            int64[] calldata thresholds,
            int64[] calldata directions,
            int64[] calldata bonuses,
            uint32[] calldata corrA,
            uint32[] calldata corrB,
            int64[] calldata corrWeights,
            uint32 iterations
        ) external pure returns (int64[] memory);
    }
}

pub struct ContractClient {
    contract_address: Address,
    rpc_url: String,
}

pub struct SolidityContractClient {
    contract_address: Address,
    rpc_url: String,
}

#[derive(Debug)]
pub struct ContractResponse {
    pub score: i64,
    pub tier: i64,
    pub correlation_adjustment: i64,
    pub iterations_run: i64,
    pub gas_used: u64,
}

// convert a scoring request into the parallel arrays both contracts expect
fn prepare_call_data(request: &ScoringRequest) -> (
    Vec<i64>, Vec<i64>, Vec<i64>, Vec<i64>, Vec<i64>,
    Vec<u32>, Vec<u32>, Vec<i64>, u32,
) {
    // factors ordered by the entity's btreemap key order
    let factor_keys: Vec<&String> = request.entity.factors.keys().collect();
    let factors: Vec<i64> = factor_keys.iter().map(|k| request.entity.factors[*k]).collect();

    let mut weights = Vec::new();
    let mut thresholds = Vec::new();
    let mut directions = Vec::new();
    let mut bonuses = Vec::new();

    // for each rule, find its factor index
    for rule in &request.rules {
        if let Some(idx) = factor_keys.iter().position(|k| **k == rule.factor_name) {
            // pad arrays up to idx if needed
            while weights.len() <= idx {
                weights.push(0i64);
                thresholds.push(0i64);
                directions.push(1i64);
                bonuses.push(0i64);
            }
            weights[idx] = rule.weight;
            thresholds[idx] = rule.threshold;
            directions[idx] = match rule.direction {
                shared::Direction::Above => 1,
                shared::Direction::Below => 0,
            };
            bonuses[idx] = rule.penalty_or_bonus;
        }
    }

    // pad to match factors length
    while weights.len() < factors.len() {
        weights.push(0);
        thresholds.push(0);
        directions.push(1);
        bonuses.push(0);
    }

    let mut corr_a = Vec::new();
    let mut corr_b = Vec::new();
    let mut corr_weights = Vec::new();

    for pair in &request.correlations {
        if let (Some(a), Some(b)) = (
            factor_keys.iter().position(|k| **k == pair.factor_a),
            factor_keys.iter().position(|k| **k == pair.factor_b),
        ) {
            corr_a.push(a as u32);
            corr_b.push(b as u32);
            corr_weights.push(pair.weight);
        }
    }

    let iterations = if request.iterations == 0 { 10 } else { request.iterations };

    (factors, weights, thresholds, directions, bonuses, corr_a, corr_b, corr_weights, iterations)
}

fn parse_result(data: Vec<i64>, gas: u64) -> ContractResponse {
    ContractResponse {
        score: *data.first().unwrap_or(&0),
        tier: *data.get(1).unwrap_or(&0),
        correlation_adjustment: *data.get(2).unwrap_or(&0),
        iterations_run: *data.get(3).unwrap_or(&0),
        gas_used: gas,
    }
}

impl ContractClient {
    pub fn new(contract_address: &str, rpc_url: &str) -> Self {
        Self {
            contract_address: Address::from_str(contract_address).expect("invalid address"),
            rpc_url: rpc_url.to_string(),
        }
    }

    pub fn address(&self) -> Address {
        self.contract_address
    }

    pub async fn evaluate(&self, request: &ScoringRequest) -> eyre::Result<ContractResponse> {
        let url: alloy::transports::http::reqwest::Url = self.rpc_url.parse()?;
        let provider = ProviderBuilder::new().on_http(url);
        let contract = IScoringEngine::new(self.contract_address, &provider);

        let (factors, weights, thresholds, directions, bonuses, corr_a, corr_b, corr_weights, iterations) =
            prepare_call_data(request);

        let call = contract.evaluate(factors, weights, thresholds, directions, bonuses, corr_a, corr_b, corr_weights, iterations);

        let gas_estimate = call.estimate_gas().await?;
        let response = call.call().await?;

        Ok(parse_result(response._0, gas_estimate))
    }
}

impl SolidityContractClient {
    pub fn new(contract_address: &str, rpc_url: &str) -> Self {
        Self {
            contract_address: Address::from_str(contract_address).expect("invalid address"),
            rpc_url: rpc_url.to_string(),
        }
    }

    pub async fn evaluate(&self, request: &ScoringRequest) -> eyre::Result<ContractResponse> {
        let url: alloy::transports::http::reqwest::Url = self.rpc_url.parse()?;
        let provider = ProviderBuilder::new().on_http(url);
        let contract = IScoringEngineSol::new(self.contract_address, &provider);

        let (factors, weights, thresholds, directions, bonuses, corr_a, corr_b, corr_weights, iterations) =
            prepare_call_data(request);

        let call = contract.evaluate(factors, weights, thresholds, directions, bonuses, corr_a, corr_b, corr_weights, iterations);

        let gas_estimate = call.estimate_gas().await?;
        let response = call.call().await?;

        Ok(parse_result(response._0, gas_estimate))
    }
}

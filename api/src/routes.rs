use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use shared::{CorrelationPair, Entity, Rule, ScoringRequest, ScoringResult};
use std::sync::Arc;
use std::time::Instant;

use crate::chain::{ContractClient, ContractResponse, SolidityContractClient};
use crate::db::Db;
use crate::scoring;

pub struct AppState {
    pub db: Db,
    pub contract: ContractClient,
    pub solidity_contract: Option<SolidityContractClient>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub contract_address: String,
}

#[derive(Deserialize)]
pub struct ScoreRequest {
    pub entity: Entity,
    #[serde(default)]
    pub correlations: Vec<CorrelationPair>,
    #[serde(default = "default_iterations")]
    pub iterations: u32,
}

fn default_iterations() -> u32 {
    10
}

#[derive(Serialize)]
pub struct BenchmarkEntry {
    pub label: String,
    pub score: i64,
    pub tier: i64,
    pub correlation_adjustment: i64,
    pub iterations_run: i64,
    pub duration_us: u128,
    pub gas_used: Option<u64>,
}

#[derive(Serialize)]
pub struct BenchmarkResponse {
    pub benchmarks: Vec<BenchmarkEntry>,
}

#[derive(Serialize)]
pub struct CreateRuleResponse {
    pub id: i64,
}

fn entry_from_contract(label: &str, resp: ContractResponse, duration: u128) -> BenchmarkEntry {
    BenchmarkEntry {
        label: label.to_string(),
        score: resp.score,
        tier: resp.tier,
        correlation_adjustment: resp.correlation_adjustment,
        iterations_run: resp.iterations_run,
        duration_us: duration,
        gas_used: Some(resp.gas_used),
    }
}

fn entry_from_offchain(result: &ScoringResult, duration: u128) -> BenchmarkEntry {
    BenchmarkEntry {
        label: "offchain_rust".to_string(),
        score: result.score,
        tier: match result.tier {
            shared::RiskTier::Low => 0,
            shared::RiskTier::Medium => 1,
            shared::RiskTier::High => 2,
            shared::RiskTier::Critical => 3,
        },
        correlation_adjustment: result.correlation_adjustment,
        iterations_run: result.iterations_run as i64,
        duration_us: duration,
        gas_used: None,
    }
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        contract_address: state.contract.address().to_string(),
    })
}

pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    Json(rule): Json<Rule>,
) -> impl IntoResponse {
    match state.db.insert_rule(&rule) {
        Ok(id) => (StatusCode::CREATED, Json(CreateRuleResponse { id })).into_response(),
        Err(e) => {
            tracing::error!("db insert error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn list_rules(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.db.list_rules() {
        Ok(rules) => Json(rules).into_response(),
        Err(e) => {
            tracing::error!("db list error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn score(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ScoreRequest>,
) -> impl IntoResponse {
    let rules = match state.db.list_rules() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("db list error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let scoring_request = ScoringRequest {
        entity: body.entity,
        rules,
        correlations: body.correlations,
        iterations: body.iterations,
    };

    match state.contract.evaluate(&scoring_request).await {
        Ok(resp) => Json(serde_json::json!({
            "score": resp.score,
            "tier": resp.tier,
            "correlation_adjustment": resp.correlation_adjustment,
        })).into_response(),
        Err(e) => {
            tracing::error!("contract call error: {e}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub async fn benchmark(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ScoreRequest>,
) -> impl IntoResponse {
    let rules = match state.db.list_rules() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("db list error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let scoring_request = ScoringRequest {
        entity: body.entity,
        rules,
        correlations: body.correlations,
        iterations: body.iterations,
    };

    let mut benchmarks = Vec::new();

    // offchain rust
    let start = Instant::now();
    let offchain_result = scoring::evaluate(&scoring_request);
    let duration = start.elapsed().as_micros();
    benchmarks.push(entry_from_offchain(&offchain_result, duration));

    // onchain stylus
    let start = Instant::now();
    match state.contract.evaluate(&scoring_request).await {
        Ok(resp) => {
            let duration = start.elapsed().as_micros();
            benchmarks.push(entry_from_contract("onchain_stylus", resp, duration));
        }
        Err(e) => {
            tracing::error!("stylus contract call error: {e}");
        }
    }

    // onchain solidity
    if let Some(sol_client) = &state.solidity_contract {
        let start = Instant::now();
        match sol_client.evaluate(&scoring_request).await {
            Ok(resp) => {
                let duration = start.elapsed().as_micros();
                benchmarks.push(entry_from_contract("onchain_solidity", resp, duration));
            }
            Err(e) => {
                tracing::error!("solidity contract call error: {e}");
            }
        }
    }

    Json(BenchmarkResponse { benchmarks }).into_response()
}

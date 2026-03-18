mod chain;
mod db;
mod routes;
mod scoring;

use axum::{
    routing::{get, post},
    Router,
};
use routes::AppState;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::from_path("../.env").ok();
    tracing_subscriber::fmt::init();

    let contract_address = std::env::var("CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8547".to_string());
    let db_path = std::env::var("DB_PATH")
        .unwrap_or_else(|_| "scoring.db".to_string());

    let solidity_contract = std::env::var("SOLIDITY_CONTRACT_ADDRESS").ok().map(|addr| {
        chain::SolidityContractClient::new(&addr, &rpc_url)
    });

    let state = Arc::new(AppState {
        db: db::Db::new(&db_path),
        contract: chain::ContractClient::new(&contract_address, &rpc_url),
        solidity_contract,
    });

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/rules", post(routes::create_rule))
        .route("/rules", get(routes::list_rules))
        .route("/score", post(routes::score))
        .route("/benchmark", post(routes::benchmark))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", port);
    axum::serve(listener, app).await.unwrap();
}

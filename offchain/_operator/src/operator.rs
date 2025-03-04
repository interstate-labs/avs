use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ethers::{
    providers::{Provider, Http, Middleware},
    types::{H256, U256},
};
use crate::PrivateKeySigner;
use k256::ecdsa::SigningKey;
use tracing::info;

// Request and response types
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub transaction_hash: String,
    pub block_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub is_included: bool,
    pub proposer_index: Option<u64>,
    pub block_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BeaconApiResponse {
    status: String,
    data: Vec<BlockData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockData {
    #[serde(rename = "posConsensus")] 
    pos_consensus: PosConsensus,
}

#[derive(Debug, Serialize, Deserialize)]
struct PosConsensus {
    #[serde(rename = "proposerIndex")]
    proposer_index: u64,
    #[serde(rename = "executionBlockNumber")]
    execution_block_number: u64,
    slot: u64,
    epoch: u64,
    finalized: bool,
}

// Application state
#[derive(Clone)]
pub struct AppState {
    provider: Arc<Provider<Http>>,
    client: Arc<reqwest::Client>,
}

impl AppState {
    pub fn new(rpc_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| eyre::eyre!("Failed to create provider: {}", e))?;

        Ok(Self {
            provider: Arc::new(provider),
            client: Arc::new(reqwest::Client::new()),
        })
    }
}

// Transaction verification functions
async fn is_transaction_in_block(
    provider: &Provider<Http>,
    tx_hash: &str,
    block_number: &str,
) -> Result<bool> {
    let tx_hash = tx_hash.parse::<H256>()?;
    
    let tx = provider
        .get_transaction(tx_hash)
        .await?;
        info!("tx     {:?}",tx);
        info!("block_number     {:?}",block_number);
        info!("tx_hash     {:?}",tx_hash);


    match tx {
        Some(tx) => {
            let tx_block_number = tx.block_number.unwrap_or_default();
            let expected_block = U256::from_dec_str(block_number)?;
            info!("expected_block     {:?}",expected_block);
            info!("tx_block_number     {:?}",tx_block_number);
            Ok(tx_block_number.as_u64() == expected_block.as_u64())
        }
        None => Ok(false),
    }
}

async fn get_block_proposer(
    client: &reqwest::Client,
    block_number: &str,
) -> Result<Option<u64>> {
    let url = format!(
        "https://beaconcha.in/api/v1/execution/block/{}",
        block_number
    );

    info!("url {:?}", url);

    let response = client
        .get(&url)
        .send()
        .await?;

    if !response.status().is_success() {
        info!("Failed to get response from beaconcha.in: {}", response.status());
        return Ok(None);
    }

    let response_text = response.text().await?;
    info!("Response text: {}", response_text);

    match serde_json::from_str::<BeaconApiResponse>(&response_text) {
        Ok(beacon_response) => {
            // Get the proposer_index from the first block's posConsensus data
            let proposer_index = beacon_response
                .data
                .first()
                .map(|block| block.pos_consensus.proposer_index);
            
            info!("Found proposer_index: {:?}", proposer_index);
            Ok(proposer_index)
        }
        Err(e) => {
            info!("Failed to parse beacon response: {}", e);
            Ok(None)
        }
    }
}


// API handlers
async fn verify_transaction(
    State(state): State<AppState>,
    Json(request): Json<VerificationRequest>,
) -> Result<Json<VerificationResponse>, String> {
    let is_included = is_transaction_in_block(
        &state.provider,
        &request.transaction_hash,
        &request.block_number,
    )
    .await
    .map_err(|e| e.to_string())?;


    info!("is_included   {:?}",is_included);

    let proposer_index = if is_included {
        info!("checking ");
        get_block_proposer(&state.client, &request.block_number)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };



    info!("is_included {:?}, {:?}",is_included,proposer_index  );


    Ok(Json(VerificationResponse {
        is_included,
        proposer_index,
        block_number: request.block_number,
    }))
}

// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}



// Router setup
pub fn operator_router(wallet: PrivateKeySigner) -> Router {
    let state = AppState::new("https://eth.llamarpc.com")
        .expect("Failed to create app state");
        
    Router::new()
        .route("/verify", post(verify_transaction))
        .route("/health", get(health_check))
        .with_state(state)
}


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
    
    // Define retry parameters
    let max_retries = 5;
    let initial_delay_ms = 1000; // 1 second
    let mut retry_count = 0;
    let mut tx = None;
    
    // Retry loop
    while retry_count < max_retries {
        info!("Attempt {} to retrieve transaction {:?}", retry_count + 1, tx_hash);
        
        // Try to get the transaction
        match provider.get_transaction(tx_hash).await {
            Ok(Some(transaction)) => {
                // Success - transaction found
                tx = Some(transaction);
                break;
            },
            Ok(None) => {
                // Transaction not found, let's retry after delay
                info!("Transaction not found, retrying...");
                retry_count += 1;
                
                // Exponential backoff
                let delay_ms = initial_delay_ms * 2_u64.pow(retry_count as u32);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            },
            Err(e) => {
                // RPC error, log and retry
                info!("RPC error: {:?}, retrying...", e);
                retry_count += 1;
                
                // Exponential backoff
                let delay_ms = initial_delay_ms * 2_u64.pow(retry_count as u32);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }
        }
    }
    
    info!("tx     {:?}", tx);
    info!("block_number     {:?}", block_number);
    info!("tx_hash     {:?}", tx_hash);
    
    match tx {
        Some(tx) => {
            let tx_block_number = tx.block_number.unwrap_or_default();
            let expected_block = U256::from_dec_str(block_number)?;
            info!("expected_block     {:?}", expected_block);
            info!("tx_block_number     {:?}", tx_block_number);
            Ok(tx_block_number.as_u64() == expected_block.as_u64())
        }
        None => {
            info!("Transaction not found after {} retries", max_retries);
            Ok(false)
        }
    }
}
// async fn is_transaction_in_block(
//     provider: &Provider<Http>,
//     tx_hash: &str,
//     block_number: &str,
// ) -> Result<bool> {
//     let tx_hash = tx_hash.parse::<H256>()?;
    
//     let tx = provider
//         .get_transaction(tx_hash)
//         .await?;

//         info!("tx     {:?}",tx);
//         info!("block_number     {:?}",block_number);
//         info!("tx_hash     {:?}",tx_hash);


//     match tx {
//         Some(tx) => {
//             let tx_block_number = tx.block_number.unwrap_or_default();
//             let expected_block = U256::from_dec_str(block_number)?;
//             info!("expected_block     {:?}",expected_block);
//             info!("tx_block_number     {:?}",tx_block_number);
//             Ok(tx_block_number.as_u64() == expected_block.as_u64())
//         }
//         None => Ok(false),
//     }
// }

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

    info!("provider_verify_transaction   {:?}",state.provider);
    let verification_task = tokio::time::timeout(
        std::time::Duration::from_secs(15), // 15 second timeout
        is_transaction_in_block(
            &state.provider,
            &request.transaction_hash,
            &request.block_number,
        )
    );
    
    // Handle timeout and results
    let is_included = match verification_task.await {
        Ok(result) => result.map_err(|e| e.to_string())?,
        Err(_) => {
            info!("Transaction verification timed out after 15 seconds");
            false // Consider transaction not included if verification times out
        }
    };
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
    let state = AppState::new("https://ethereum-holesky.publicnode.com")
    .expect("Failed to create app state");
        
    Router::new()
        .route("/verify", post(verify_transaction))
        .route("/health", get(health_check))
        .with_state(state)
}


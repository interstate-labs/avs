use aggregator::aggregator_router;
use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use axum::{routing::get, Router};
use eyre::Result;
use serde::Deserialize;
use std::{fs, net::IpAddr, str::FromStr, sync::Arc};
use task::BlockNumberData;
use thiserror::Error;
use url::Url;

pub mod aggregator;
pub mod contract;
pub mod error;
pub mod health;
pub mod task;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub rpc_url: String,
    pub private_key: String,
    pub square_number_dss_address: Address,
    pub txn_verifier_address:Address,
    pub core_address: Address,
    pub block_number_store: String,
    pub heartbeat: u64,
}

#[derive(Debug, Deserialize)]
pub struct ContractAddresses {
    pub square_number_dss: String,
    pub core: String,
    pub block_number: u64,
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Contract call error")]
    ContractCallError,

    #[error("Operator not found")]
    OperatorNotFound,

    #[error("Task verification failed")]
    TaskVerificationFailed,

    #[error("Majority not reached")]
    MajorityNotReached,

    #[error("JSON parsing error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Failed to load contract JSON: {0}")]
    LoadContractJsonError(String),

    #[error("Custom URL error: {0}")]
    CustomUrlError(String),

    #[error("NoOperatorAvailable")]
    NoOperatorAvailable,

    #[error("RequestError")]
    RequestError
}

impl Config {
    pub fn load_block_number(&self) -> Result<u64, TaskError> {
        let file_content = fs::read_to_string(&self.block_number_store)
            .map_err(|e| TaskError::LoadContractJsonError(e.to_string()))?;

        let block_number_data: BlockNumberData = serde_json::from_str(&file_content)
            .map_err(|e| TaskError::LoadContractJsonError(e.to_string()))?;

        Ok(block_number_data.block_number)
    }

    pub fn get_rpc_url(&self) -> Result<Url> {
        Ok(Url::parse(&self.rpc_url)?)
    }

    pub fn get_private_key(&self) -> Result<PrivateKeySigner, TaskError> {
        let private_key = PrivateKeySigner::from_str(&self.private_key)
            .map_err(|e| TaskError::CustomUrlError(e.to_string()))?;
        Ok(private_key)
    }
}

pub fn routes(operator_state: Arc<aggregator::OperatorState>) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .nest("/aggregator", aggregator_router(operator_state))
}

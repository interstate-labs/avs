use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use axum::{routing::get, Router};
use serde::Deserialize;
use std::{net::IpAddr, str::FromStr};
use thiserror::Error;
use url::Url;

pub mod contract;
pub mod error;
pub mod health;
pub mod operator;
pub mod register;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub rpc_url: Url,
    #[serde(deserialize_with = "deserialize_private_key")]
    pub private_key: alloy::signers::local::PrivateKeySigner,
    pub domain_url: Url,
    pub aggregator_url: Url,
    pub square_number_dss_address: Address,
     pub txn_verifier_address:Address,
    pub core_address: Address,
    pub heartbeat: u64,
}

fn deserialize_private_key<'de, D>(deserializer: D) -> Result<PrivateKeySigner, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    PrivateKeySigner::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct ContractAddresses {
    pub square_number_dss: Address,
    pub core: Address,
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

    #[error("Signature Conversion error")]
    SignatureConversionError,

    #[error("JSON parsing error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Custom URL error: {0}")]
    CustomUrlError(String),

    #[error("Signing error: {0}")]
    SigningError(#[from] alloy::signers::Error),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Error looking up transaction: {0}")]
    TransactionLookupError(String),
    
    #[error("Error looking up proposer: {0}")]
    ProposerLookupError(String),
}

pub fn routes(wallet: PrivateKeySigner) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .nest("/operator", operator::operator_router(wallet))
}

use alloy::primitives::Address;
use axum::{extract::State, routing::post, Json, Router};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};
use tracing::info;
use url::Url;

use crate::error::AppError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    public_key: Address,
    url: Url,
}

impl Operator {
    pub fn new(public_key: Address, url: Url) -> Self {
        Self { public_key, url }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn public_key(&self) -> &Address {
        &self.public_key
    }
}

#[derive(Clone, Debug, Default)]
pub struct OperatorState {
    pub operators: Arc<RwLock<HashSet<Operator>>>,
}

impl OperatorState {
    pub fn new() -> Self {
        Self {
            operators: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn register_operator(&self, operator: Operator) -> Result<()> {
        let mut operators = self
            .operators
            .write()
            .map_err(|_| eyre::eyre!("Could not lock"))?;
        if operators.insert(operator.clone()) {
            info!("Operator registered: {}", serde_json::to_string(&operator)?);
        } else {
            info!(
                "Operator already registered: {}",
                serde_json::to_string(&operator)?
            );
        }
        Ok(())
    }

    pub fn is_operator_registered(&self, operator: Operator) -> Result<bool> {
        Ok(self
            .operators
            .read()
            .map_err(|_| eyre::eyre!("Could not lock"))?
            .contains(&operator))
    }
}

pub async fn register_operator(
    State(operators): State<Arc<OperatorState>>,
    Json(operator): Json<Operator>,
) -> Result<Json<bool>, AppError> {
    operators.register_operator(operator)?;
    Ok(Json(true))
}

pub async fn is_operator_registered(
    State(operators): State<Arc<OperatorState>>,
    Json(operator): Json<Operator>,
) -> Result<Json<bool>, AppError> {
    let registered = operators.is_operator_registered(operator)?;
    Ok(Json(registered))
}

pub fn aggregator_router(operator_state: Arc<OperatorState>) -> Router {
    Router::new()
        .route("/registerOperator", post(register_operator))
        .route("/isOperatorRegistered", post(is_operator_registered))
        .with_state(operator_state)
}

pub mod abi;
// In lib.rs
pub use crate::abi::EigenLayerRestaking;
// use crate::abi::SignatureWithSaltAndExpiry;
use crate::abi::SignatureWithSaltAndExpiry;
use anyhow::Result;
use ethers::{
    prelude::*,
    types::{Address, Bytes, H256, U256},
    middleware::SignerMiddleware,
};
pub mod event_fetcher;
use std::{env, sync::Arc, str::FromStr};

// mod abi;
// Import the type directly from the module
// use crate::abi::EigenLayerRestaking;

pub struct EigenLayerClient {
    contract: EigenLayerRestaking<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl EigenLayerClient {
    pub fn new(contract_address: Address) -> Result<Self> {
        // Use Sepolia RPC URL from environment variable or default to public endpoint
        let provider_url = env::var("HOLESKY_RPC_URL")
            .unwrap_or_else(|_| "https://ethereum-holesky-rpc.publicnode.com".to_string());
        
        // Get private key from environment variable
        let private_key = env::var("PRIVATE_KEY")
            .expect("PRIVATE_KEY environment variable must be set");
        
        // Sepolia chain ID is 11155111
        let chain_id: u64 = 17000;
        
        let provider = Provider::<Http>::try_from(provider_url)?;
        let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        
        // Create a SignerMiddleware with provider and wallet
        let client = SignerMiddleware::new(provider, wallet);
        
        let contract = EigenLayerRestaking::new(
            contract_address,
            Arc::new(client),
        );

        Ok(Self { contract })
    }

    // Get the interface version
    pub async fn get_upgrade_interface_version(&self) -> Result<String> {
        Ok(self.contract.upgrade_interface_version().call().await?)
    }

    // Get AVS directory address
    pub async fn get_avs_directory(&self) -> Result<Address> {
        Ok(self.contract.avs_directory().call().await?)
    }

    // Deregister an operator from AVS
    pub async fn deregister_operator_from_avs(&self, operator: Address) -> Result<()> {
        let tx = self.contract.deregister_operator_from_avs(operator);
        let pending_tx = tx.send().await?;
        println!("Operator deregistered: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Deregister a strategy
    pub async fn deregister_strategy(&self, strategy: Address) -> Result<()> {
        let tx = self.contract.deregister_strategy(strategy);
        let pending_tx = tx.send().await?;
        println!("Strategy deregistered: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Get current period
    // pub async fn get_current_period(&self) -> Result<u64> {
    //     let period = self.contract.get_current_period().call().await?;
    //     Ok(period.as_u64())
    // }
    // Get operator restaked strategies
    pub async fn get_operator_restaked_strategies(&self, operator: Address) -> Result<Vec<Address>> {
        Ok(self.contract.get_operator_restaked_strategies(operator).call().await?)
    }

    // Get operator stake at a specific timestamp
    pub async fn get_operator_stake_at(
        &self,
        operator: Address,
        collateral: Address,
        timestamp: u64,
    ) -> Result<U256> {
        Ok(self.contract
            .get_operator_stake_at(operator, collateral, timestamp as u64)
            .call()
            .await?)
    }

    // Get provider collateral
    pub async fn get_provider_collateral(
        &self,
        operator: Address,
        collateral: Address,
    ) -> Result<U256> {
        Ok(self.contract
            .get_provider_collateral(operator, collateral)
            .call()
            .await?)
    }

    // Get provider collateral tokens
    pub async fn get_provider_collateral_tokens(
        &self,
        operator: Address,
    ) -> Result<(Vec<Address>, Vec<U256>)> {
        Ok(self.contract
            .get_provider_collateral_tokens(operator)
            .call()
            .await?)
    }

    // Get restakeable strategies
    pub async fn get_restakeable_strategies(&self) -> Result<Vec<Address>> {
        Ok(self.contract.get_restakeable_strategies().call().await?)
    }

    // Get whitelisted strategies
    pub async fn get_whitelisted_strategies(&self) -> Result<Vec<Address>> {
        Ok(self.contract.get_whitelisted_strategies().call().await?)
    }

    // Initialize the contract
    pub async fn initialize(
        &self,
        owner: Address,
        parameters: Address,
        eigenlayer_avs_directory: Address,
        eigenlayer_delegation_manager: Address,
        eigenlayer_strategy_manager: Address,
        restaking_helper: Address,
    ) -> Result<()> {
        let tx = self.contract.initialize(
            owner,
            parameters,
            eigenlayer_avs_directory,
            eigenlayer_delegation_manager,
            eigenlayer_strategy_manager,
            restaking_helper,
        );
        let pending_tx = tx.send().await?;
        println!("Contract initialized: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Check if a strategy is enabled
    pub async fn is_strategy_enabled(&self, strategy: Address) -> Result<bool> {
        Ok(self.contract.is_strategy_enabled(strategy).call().await?)
    }

    // Get the contract owner
    pub async fn get_owner(&self) -> Result<Address> {
        Ok(self.contract.owner().call().await?)
    }

    // Pause strategy
    pub async fn pause_strategy(&self) -> Result<()> {
        let tx = self.contract.pause_strategy();
        let pending_tx = tx.send().await?;
        println!("Strategy paused: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Get proxiable UUID
    pub async fn get_proxiable_uuid(&self) -> Result<[u8; 32]> {
        Ok(self.contract.proxiable_uuid().call().await?)
    }

    // Register operator
    pub async fn register_operator(
        &self,
        rpc: String,
        rpc1: String,
        rpc2: String,
        signature: Vec<u8>,
        salt: [u8; 32],
        expiry: U256,
    ) -> Result<()> {
        let operator_signature = SignatureWithSaltAndExpiry {
            signature: Bytes::from(signature),
            salt,
            expiry,
        };

        let tx = self.contract.register_operator(rpc, rpc1, rpc2, operator_signature);
        let pending_tx = tx.send().await?;
        println!("Operator registered: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Register operator to AVS
    pub async fn register_operator_to_avs(
        &self,
        operator: Address,
        signature: Vec<u8>,
        salt: [u8; 32],
        expiry: U256,
    ) -> Result<()> {
        let operator_signature = SignatureWithSaltAndExpiry {
            signature: Bytes::from(signature),
            salt,
            expiry,
        };

        let tx = self.contract.register_operator_to_avs(operator, operator_signature);
        let pending_tx = tx.send().await?;
        println!("Operator registered to AVS: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Register strategy
    pub async fn register_strategy(&self, strategy: Address) -> Result<()> {
        let tx = self.contract.register_strategy(strategy);
        let pending_tx = tx.send().await?;
        println!("Strategy registered: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Renounce ownership
    pub async fn renounce_ownership(&self) -> Result<()> {
        let tx = self.contract.renounce_ownership();
        let pending_tx = tx.send().await?;
        println!("Ownership renounced: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Get restaking helper address
    pub async fn get_restaking_helper(&self) -> Result<Address> {
        Ok(self.contract.restaking_helper().call().await?)
    }

    // Transfer ownership
    pub async fn transfer_ownership(&self, new_owner: Address) -> Result<()> {
        let tx = self.contract.transfer_ownership(new_owner);
        let pending_tx = tx.send().await?;
        println!("Ownership transferred: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Unpause strategy
    pub async fn unpause_strategy(&self) -> Result<()> {
        let tx = self.contract.unpause_strategy();
        let pending_tx = tx.send().await?;
        println!("Strategy unpaused: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Update AVS metadata URI
    pub async fn update_avs_metadata_uri(&self, metadata_uri: String) -> Result<()> {
        let tx = self.contract.update_avs_metadata_uri(metadata_uri);
        let pending_tx = tx.send().await?;
        println!("AVS metadata URI updated: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Upgrade to new implementation
    pub async fn upgrade_to_and_call(
        &self,
        new_implementation: Address,
        data: Vec<u8>,
    ) -> Result<()> {
        let tx = self.contract.upgrade_to_and_call(
            new_implementation,
            Bytes::from(data),
        );
        
        // Note: This is a payable function, you might need to specify value
        let pending_tx = tx.send().await?;
        println!("Contract upgraded: {:?}", pending_tx.tx_hash());
        Ok(())
    }
}
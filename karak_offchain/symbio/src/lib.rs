use anyhow::Result;
use ethers::{
    prelude::*,
    types::{Address, Bytes, H256},
    middleware::SignerMiddleware,
};
use std::{env, sync::Arc};

 mod abi;
// Import the type directly from the module
use crate::abi::SymbioticRestaking;



pub struct SymbioticClient {
    contract: SymbioticRestaking<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl SymbioticClient {
    pub fn new(
        contract_address: Address,
    ) -> Result<Self> {
        // Use Sepolia RPC URL from environment variable or default to public endpoint
        let provider_url = env::var("SEPOLIA_RPC_URL")
            .unwrap_or_else(|_| "https://rpc.sepolia.org".to_string());
        
        // Get private key from environment variable
        let private_key = env::var("PRIVATE_KEY")
            .expect("PRIVATE_KEY environment variable must be set");
        
        // Sepolia chain ID is 11155111
        let chain_id: u64 = 11155111;
        
        let provider = Provider::<Http>::try_from(provider_url)?;
        let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        
        // Create a SignerMiddleware with provider and wallet
        let client = SignerMiddleware::new(
            provider,
            wallet,
        );
        
        let contract = SymbioticRestaking::new(
            contract_address,
            Arc::new(client),
        );

        Ok(Self { contract })
    }
    // Get whitelisted vaults
    pub async fn get_whitelisted_vaults(&self) -> Result<Vec<Address>> {
        Ok(self.contract.get_whitelisted_vaults().call().await?)
    }

    // Add this to your SymbioticClient implementation
pub async fn initialize(
    &self,
    owner: Address,
    parameters: Address,
    symbiotic_network: Address,
    symbiotic_operator_registry: Address,
    symbiotic_operator_net_opt_in: Address,
    symbiotic_vault_factory: Address,
) -> Result<()> {
    let tx = self.contract.initialize(
        owner,
        parameters,
        symbiotic_network,
        symbiotic_operator_registry,
        symbiotic_operator_net_opt_in,
        symbiotic_vault_factory,
    );
    let pending_tx = tx.send().await?;
    println!("Contract initialized: {:?}", pending_tx.tx_hash());
    Ok(())
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

    // Submit slash request
    pub async fn slash(
        &self,
        validator_pubkey: String,
        block_number: u64,
        tx_id: H256,
    ) -> Result<()> {
        let tx = self.contract.slash(validator_pubkey, block_number.into(), tx_id.into());
        let pending_tx = tx.send().await?;
        println!("Transaction submitted: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Check validator response
    pub async fn get_validator_response(
        &self,
        validator_pubkey: String,
        block_number: u64,
        tx_id: H256,
    ) -> Result<bool> {
        Ok(self.contract
            .get_validator_response(validator_pubkey, block_number.into(), tx_id.into())
            .call()
            .await?)
    }

    // Register operator
    pub async fn register_operator(&self, operator_addr: Address, rpc: String) -> Result<()> {
        let tx = self.contract.register_operator(operator_addr, rpc);
        let pending_tx = tx.send().await?;
        println!("Operator registered: {:?}", pending_tx.tx_hash());
        Ok(())
    }

    // Check vault status
    pub async fn is_vault_enabled(&self, vault: Address) -> Result<bool> {
        Ok(self.contract.is_vault_enabled(vault).call().await?)
    }

    // Get current epoch
    pub async fn get_current_time(&self) -> Result<u64> {
        Ok(self.contract.get_current_time().call().await?.into())
    }
}
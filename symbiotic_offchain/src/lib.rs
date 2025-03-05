use anyhow::Result;
use ethers::{
    prelude::*,
    types::{Address, Bytes, H256},
    middleware::SignerMiddleware,
};
use std::{ sync::Arc};

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
        let provider_url = "https://rpc.sepolia.org";
        
        // Get private key from environment variable
        let private_key = "eaaa2702cad0c15b0cd341673c46d6936f87be6c3334c41a616201ea6931297a";
        
        // Sepolia chain ID is 11155111
        let chain_id: u64 = 17000;
        
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

    // Helper to get contract reference
    pub async fn get_contract(&self) -> Result<&SymbioticRestaking<SignerMiddleware<Provider<Http>, LocalWallet>>> {
        Ok(&self.contract)
    }

    // Get whitelisted vaults
    pub async fn get_whitelisted_vaults(&self) -> Result<Vec<Address>> {
        Ok(self.contract.get_whitelisted_vaults().call().await?)
    }

    // Initialize contract
    pub async fn initialize(
        &self,
        parameters: Address,
        symbiotic_network: Address,
        symbiotic_operator_registry: Address,
        symbiotic_operator_net_opt_in: Address,
        symbiotic_vault_factory: Address,
    ) -> Result<()> {
        let tx = self.contract.initialize(
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

    // Add a new owner
    pub async fn add_owner(&self, owner_address: Address) -> Result<()> {
        let tx = self.contract.add_owner(owner_address);
        let pending_tx = tx.send().await?;
        let receipt = pending_tx.await?;

        Ok(())
    }

    // Initiate owner removal (with 1-minute delay)
    pub async fn remove_owner(&self, owner_address: Address) -> Result<()> {
        println!("remove_owner: {:?}", owner_address);
        let tx = self.contract.remove_owner(owner_address);
        println!("tx: {:?}", tx);
        let pending_tx = tx.send().await?;
        println!("pending_tx: {:?}", pending_tx);
        let receipt = pending_tx.await?;
        println!("remove_owner_receipt: {:?}", receipt);

        Ok(())
    }

    // Execute pending owner removal after delay
    pub async fn execute_remove_owner(&self, owner_address: Address) -> Result<()> {
        println!("execute_remove_owner: {:?}", owner_address);
        let tx = self.contract.execute_remove_owner(owner_address);
        println!("tx: {:?}", tx);
        let pending_tx = tx.send().await?;
        let receipt = pending_tx.await?;
        println!("execute_remove_ownerreceipt: {:?}", receipt);

        Ok(())
    }

    // Check if an address is an owner
    pub async fn is_owner(&self, owner_address: Address) -> Result<bool> {
        Ok(self.contract.is_owner(owner_address).call().await?)
    }

    // Get time remaining for a pending owner removal
    pub async fn get_remove_owner_time_remaining(&self, owner_address: Address) -> Result<U256> {
        Ok(self.contract.get_remove_owner_time_remaining(owner_address).call().await?)
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

    // Verify transaction result
    pub async fn verified_txn(
        &self,
        result: bool,
        validator_pubkey: String,
        block_number: u64,
        tx_id: H256,
    ) -> Result<()> {
        let tx = self.contract.verified_txn(result, validator_pubkey, block_number.into(), tx_id.into());
        let pending_tx = tx.send().await?;
        let receipt = pending_tx.await?;
        Ok(())
    }
}
mod abi;
mod event_fetcher;

use abi::SymbioticRestaking;
use event_fetcher::EventFetcher;
use symbio::SymbioticClient;
use ethers::{
    prelude::*,
    providers::{Provider, Http},

};

use anyhow::Result;
use clap::{Parser, Subcommand};
use ethers::types::{Address, H256};

use std::{str::FromStr, sync::Arc};

#[derive(Parser)]
#[command(name = "Symbiotic Restaking CLI")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get whitelisted vaults
    GetVaults,

    /// Get provider collateral
    GetCollateral {
        operator: String,
        collateral: String,
    },

    /// Submit slash request
    Slash {
        validator_pubkey: String,
        block_number: u64,
        tx_id: String,
    },

    /// Check validator response
    CheckResponse {
        validator_pubkey: String,
        block_number: u64,
        tx_id: String,
    },

    /// Register operator
    RegisterOperator {
        operator_addr: String,
        rpc: String,
    },

    /// Check vault status
    CheckVault {
        vault: String,
    },

    /// Get current epoch
    GetCurrentEpoch,

    FetchEvents,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let client = SymbioticClient::new(dotenv::var("CONTRACT_ADDRESS")?.parse()?)?;

    match cli.command {
        Commands::GetVaults => {
            let vaults = client.get_whitelisted_vaults().await?;
            println!("Whitelisted vaults: {:?}", vaults);
        }

        Commands::GetCollateral {
            operator,
            collateral,
        } => {
            let amount = client
                .get_provider_collateral(
                    Address::from_str(&operator)?,
                    Address::from_str(&collateral)?,
                )
                .await?;
            println!("Collateral amount: {}", amount);
        }

        Commands::Slash {
            validator_pubkey,
            block_number,
            tx_id,
        } => {
            client
                .slash(validator_pubkey, block_number, H256::from_str(&tx_id)?)
                .await?;
        }

        Commands::CheckResponse {
            validator_pubkey,
            block_number,
            tx_id,
        } => {
            let response = client
                .get_validator_response(
                    validator_pubkey,
                    block_number,
                    H256::from_str(&tx_id)?,
                )
                .await?;
            println!("Validator response: {}", response);
        }

        Commands::RegisterOperator { operator_addr, rpc } => {
            client
                .register_operator(Address::from_str(&operator_addr)?, rpc)
                .await?;
        }

        Commands::CheckVault { vault } => {
            let status = client.is_vault_enabled(Address::from_str(&vault)?).await?;
            println!("Vault enabled status: {}", status);
        }

        Commands::GetCurrentEpoch => {
            let epoch = client.get_current_time().await?;
            println!("Current epoch: {}", epoch);
        }

        Commands::FetchEvents => {
            // let contract_address = dotenv::var("CONTRACT_ADDRESS")?.parse()?;
            let contract_address = Address::from_str(&std::env::var("CONTRACT_ADDRESS")?)?;
    
            // let rpc_url = dotenv::var("ETHEREUM_RPC_URL")?;
            let rpc_url = std::env::var("ETHEREUM_RPC_URL")?;
            let provider = Provider::<Http>::try_from(rpc_url.clone())?;
    
            println!("rpc_url {} contract_address{} ",rpc_url,contract_address);
            let private_key = std::env::var("PRIVATE_KEY")?;
            let chain_id: u64 = 11155111; 
            let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
    

            let client = SignerMiddleware::new(provider, wallet);

            let contract = SymbioticRestaking::new(contract_address, Arc::new(client));



            let event_fetcher = EventFetcher::new(&rpc_url, contract_address)?;
            event_fetcher.start_continuous_fetching(&contract,&rpc_url).await?;
        }
    }

    Ok(())
}

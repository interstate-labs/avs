use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::Address;
use alloy::providers::fillers::{FillProvider, JoinFill, RecommendedFiller, WalletFiller};
use alloy::providers::{ProviderBuilder, ReqwestProvider};
use alloy::sol;
use alloy::transports::http::ReqwestTransport;
use karak_rs::contracts::Core::CoreInstance;
use url::Url;
use SquareNumberDSS::{TaskRequest, TaskResponse};
use TxnVerifier::{Task,OperatorResponse};
use tracing::error;
use tracing::info;

use crate::Config;
use crate::TaskError;

sol!(
    #[sol(rpc)]
    SquareNumberDSS,
    "../abi/SquareNumberDSS.json",
);


sol!(
    #[sol(rpc)]
    TxnVerifier,
    "../abi/TxnVerifier.json",
);

sol!(
    #[sol(rpc)]
    #[allow(clippy::too_many_arguments)]
    VaultAbi,
    "../abi/Vault.json",
);

type RecommendedProvider = FillProvider<
    JoinFill<RecommendedFiller, WalletFiller<EthereumWallet>>,
    ReqwestProvider,
    ReqwestTransport,
    Ethereum,
>;

pub struct ContractManager {
    pub dss_instance:
    TxnVerifier::TxnVerifierInstance<ReqwestTransport, RecommendedProvider>,
    pub core_instance: CoreInstance<ReqwestTransport, RecommendedProvider>,
    pub provider: RecommendedProvider,
}

impl ContractManager {
    pub fn new(config: &Config) -> Result<Self, TaskError> {
        let rpc_url = config
            .get_rpc_url()
            .map_err(|e| TaskError::CustomUrlError(e.to_string()))?;
        let private_key = config.get_private_key()?;

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(private_key))
            .on_http(rpc_url);

        let txn_verifier_address = config.txn_verifier_address;
        let dss_instance = TxnVerifier::new(txn_verifier_address, provider.clone());

        let core_address = config.core_address;
        let core_instance = CoreInstance::new(core_address, provider.clone());

        Ok(Self {
            dss_instance,
            core_instance,
            provider,
        })
    }

    pub async fn fetch_vaults_staked_in_dss(
        &self,
        operator: Address,
        dss_address: Address,
    ) -> Result<Vec<Address>, TaskError> {
        let result = self
            .core_instance
            .fetchVaultsStakedInDSS(operator, dss_address)
            .call()
            .await
            .map_err(|_| TaskError::ContractCallError)?;

        Ok(result.vaults)
    }

    pub async fn submit_task_response(
        &self,
        dss_task_request: Task,
        task_response: OperatorResponse,
    ) -> Result<(), TaskError> {


        info!("submit_task_responseafter  {:?}",dss_task_request.pubkey);
        info!("dss_task_request.txn  {:?}",dss_task_request.transaction_hash);
        info!("task_response.txn  {:?}",task_response.is_included);
        info!("task_response.txn  {:?}",task_response.proposer_index);

        let contract_response = (
            task_response.is_included,
            task_response.proposer_index,
            task_response.block_number,
        );
    
        let _ = self
            .dss_instance
            .submitTaskResponse(
                dss_task_request.pubkey,
                dss_task_request.transaction_hash,
                contract_response.into()
            )
            .send()
            .await
            .map_err(|e| {
                error!("Contract call error: {:?}", e);
                TaskError::ContractCallError
            })?;
    
        // info!("Task response submitted successfully: {:?}", result.transaction_hash());

        // let _ = self
        //     .dss_instance
        //     .submitTaskResponse(dss_task_request.pubkey,dss_task_request.transaction_hash, task_response)
        //     .send()
        //     .await
        //     .map_err(|_| TaskError::ContractCallError)?;

        Ok(())
    }
}

pub struct VaultContract {
    pub vault_instance: VaultAbi::VaultAbiInstance<ReqwestTransport, RecommendedProvider>,
    pub provider: RecommendedProvider,
}

impl VaultContract {
    pub fn new(
        rpc_url: Url,
        private_key: alloy::signers::local::PrivateKeySigner,
        vault_address: Address,
    ) -> Result<Self, TaskError> {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(private_key))
            .on_http(rpc_url);

        let vault_instance = VaultAbi::new(vault_address, provider.clone());

        Ok(Self {
            vault_instance,
            provider,
        })
    }
}

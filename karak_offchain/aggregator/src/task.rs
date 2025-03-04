use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::{Arc, PoisonError},
    time::Duration,
};


use alloy::{
    primitives::{Address, Uint, U256},
    providers::Provider,
    rpc::types::{BlockNumberOrTag, Filter},
    signers::Signature,
    sol_types::SolEvent,
    transports::http::Client,
};
use num_traits::cast::ToPrimitive;
use chrono::{DateTime, Utc};
use eyre::Result;
// use crate::task::TxnVerifier::OperatorResponse;
use crate::task::SquareNumberDSS::TaskResponse;
use serde::{Deserialize, Serialize};
use tokio::{
    signal,
    time::{self},
};
use tracing::{error, info};
use url::Url;

use crate::{
    aggregator::{Operator, OperatorState},
    contract::{ContractManager, SquareNumberDSS, VaultContract,TxnVerifier},
    Config, TaskError,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub pubkey:String,
    pub transaction_hash: String,
    pub block_number: String,
}

// Struct for operator verification response
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub is_included: bool,
    pub proposer_index: Option<u64>,
    pub block_number: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task: Task,
    pub block_number: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletedTask {
    pub value: U256,
    pub response: U256,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperatorResponse {
    pub is_included: bool,
    pub proposer_index: Option<u64>,
    pub block_number: String,
}

#[derive(Serialize, Deserialize)]
pub struct BlockNumberData {
    pub block_number: u64,
}

pub struct TaskService {
    contract_manager: ContractManager,
    operator_state: Arc<OperatorState>,
    square_number_address: Address,
    dss_address: Address,
    block_number_store: String,
    block_number: u64,
    rpc_url: Url,
    private_key: alloy::signers::local::PrivateKeySigner,
    client: Client,
    txn_verifier_address:Address,
    heartbeat_interval: Duration,
}

impl TaskService {
    pub fn new(operator_state: Arc<OperatorState>, config: Config) -> Result<Self> {
        let contract_manager = ContractManager::new(&config)?;
        let square_number_address = config.txn_verifier_address;
        let dss_address = config.txn_verifier_address;
        let block_number_store = config.block_number_store.clone();
        info!("aggregato_config {:?}",config);

        let block_number: u64 = config.load_block_number()?;
        let rpc_url = config.get_rpc_url()?;
        let private_key = config.get_private_key()?;
        let txn_verifier_address=config.txn_verifier_address;
        let heartbeat_interval = Duration::from_millis(config.heartbeat);
        let client = Client::new();
        Ok(Self {
            contract_manager,
            operator_state,
            square_number_address,
            txn_verifier_address,
            dss_address,
            block_number_store,
            block_number,
            rpc_url,
            private_key,
            client,
            heartbeat_interval,
        })
    }

    pub async fn start(self: Arc<Self>) {
        info!("Listening for task request events");

        let heartbeat_interval = self.heartbeat_interval;

        tokio::spawn(async move {
            let mut interval = time::interval(heartbeat_interval);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = self.watch_for_task_events().await {
                            error!("Failed to watch for task events: {e}");
                        }
                    }
                    _ = signal::ctrl_c() => {
                        info!("Received shutdown signal. Stopping the aggregator...");
                        break;
                    }
                }
            }

            info!("Aggregator service stopped gracefully.");
        });
    }



    // async fn watch_for_task_events(&self) -> Result<()> {
    //     let txn_verifier_address = self.txn_verifier_address;
    //     let next_block_to_check: u64 = self.block_number;
        
    //     // Get current block number
    //     let current_block = match self.contract_manager.provider.get_block_number().await {
    //         Ok(block) => {
    //             info!("Current block number: {}", block);
    //             block
    //         },
    //         Err(e) => {
    //             error!("Failed to get current block number: {}", e);
    //             return Err(e.into());
    //         }
    //     };
    
    //     // Safety check for block numbers
    //     if next_block_to_check > current_block {
    //         error!("Next block to check ({}) is greater than current block ({})", 
    //                next_block_to_check, current_block);
    //         return Ok(());
    //     }
    
    //     // Create filter with specific event signature
    //     let filter = Filter::new()
    //         .address(txn_verifier_address)
    //         .from_block(BlockNumberOrTag::Number(next_block_to_check))
    //         .to_block(BlockNumberOrTag::Number(current_block));
        
    //     info!("Checking blocks {} to {} for events at address: {:?}", 
    //           next_block_to_check, current_block, txn_verifier_address);
    
    //     // Get operators with better error handling
    //     let operators = match self.operator_state.operators.read() {
    //         Ok(guard) => {
    //             let ops = guard.clone();
    //             info!("Found {} registered operators", ops.len());
    //             ops
    //         },
    //         Err(e) => {
    //             error!("Failed to acquire read lock on operator state: {}", e);
    //             return Err(eyre::anyhow!("Failed to acquire read lock on operator state"));
    //         }
    //     };
    
    //     // Get logs with timeout and retry
    //     let logs = match tokio::time::timeout(
    //         Duration::from_secs(10),
    //         self.contract_manager.provider.get_logs(&filter)
    //     ).await {
    //         Ok(result) => match result {
    //             Ok(logs) => {
    //                 info!("Retrieved {} logs from filter", logs.len());
    //                 logs
    //             },
    //             Err(e) => {
    //                 error!("Failed to get logs: {}", e);
    //                 return Err(e.into());
    //             }
    //         },
    //         Err(_) => {
    //             error!("Timeout while fetching logs");
    //             return Err(eyre::anyhow!("Timeout while fetching logs"));
    //         }
    //     };
    
    //     let mut new_last_checked_block = next_block_to_check;
    
    //     for log in logs {
    //         info!("Processing log with topics: {:?}", log.topics());
            
    //         if let Some(topic0) = log.topic0() {
    //             info!("Found event with topic0: {:?}", topic0);
                
    //             // Check if it matches your event signature
    //             if topic0 == &TxnVerifier::TxnVerificationResult::SIGNATURE_HASH {
    //                 match log.log_decode()?.inner.data {
    //                     TxnVerifier::TxnVerificationResult {
    //                         pubkey,
    //                         txnHash,
    //                         blockNumber,
    //                     } => {
    //                         info!("Decoded event data - pubkey: {:?}, txnHash: {:?}, blockNumber: {:?}",
    //                               pubkey, txnHash, blockNumber);
    
    //                         let task = Task {
    //                             pubkey: pubkey.to_string(),
    //                             transaction_hash: txnHash.to_string(),
    //                             block_number: blockNumber.to_string()
    //                         };
    
    //                         if !operators.is_empty() {
    //                             match self.send_task_to_all_operators(task, &operators).await {
    //                                 Ok(response) => {
    //                                     info!("Received operator response: {:?}", response);
                                        
    //                                     let task_response = TxnVerifier::OperatorResponse {
    //                                         is_included: response.is_included,
    //                                         proposer_index: response.proposer_index.unwrap_or(0) as u64,
    //                                         block_number: response.block_number,
    //                                     };
                                        
    //                                     let dss_task_request = TxnVerifier::Task {
    //                                         pubkey: pubkey.to_string(),
    //                                         transaction_hash: txnHash.to_string(),
    //                                         block_number: blockNumber.to_string()
    //                                     };
    
    //                                     match self.contract_manager
    //                                         .submit_task_response(dss_task_request, task_response)
    //                                         .await 
    //                                     {
    //                                         Ok(tx) => info!("Transaction submitted successfully: {:?}", tx),
    //                                         Err(e) => error!("Failed to submit task response: {:?}", e),
    //                                     }
    //                                 },
    //                                 Err(e) => error!("Failed to send task to operators: {:?}", e),
    //                             }
    //                         } else {
    //                             info!("No operators available to process the task");
    //                         }
    //                     }
    //                 }
    //             } else {
    //                 info!("Skipping non-matching event topic");
    //             }
    //         }
    
    //         if let Some(block_number) = log.block_number {
    //             new_last_checked_block = new_last_checked_block.max(block_number + 1);
    //         }
    //     }
    
    //     // Update the block number file
    //     match self.write_block_number_to_file(&self.block_number_store, new_last_checked_block).await {
    //         Ok(_) => info!("Updated block number file to {}", new_last_checked_block),
    //         Err(e) => error!("Failed to update block number file: {}", e),
    //     }
    
    //     Ok(())
    // }


    async fn watch_for_task_events(&self) -> Result<()> {

       
        let square_number_address = self.square_number_address;
        info!("square_number_address {:?}",square_number_address);
        let next_block_to_check: u64 = self.block_number;
        
        // Get current block number
        let current_block = self.contract_manager.provider.get_block_number().await?;
        
        // Process in chunks of 10000 blocks to stay well under the 50000 limit
        let chunk_size = 10000u64;
        let mut start_block = next_block_to_check;
        let mut new_last_checked_block = next_block_to_check;

        info!("new_last_checked_block {:?}",new_last_checked_block);

        info!("start_block  {:?},{:?}",start_block ,current_block);


        while start_block <= current_block {
            let end_block = (start_block + chunk_size - 1).min(current_block);
            
            let filter = Filter::new()
                .address(square_number_address)
                .from_block(BlockNumberOrTag::Number(start_block))
                .to_block(BlockNumberOrTag::Number(end_block));

                info!("filter {:?}",filter);

            let operators = match self.operator_state.operators.read() {
                Ok(guard) => guard.clone(),
                Err(PoisonError { .. }) => {
                    error!("Failed to acquire read lock on operator state");
                    return Err(eyre::anyhow!(
                        "Failed to acquire read lock on operator state"
                    ));
                }
            };

            info!("operators {:?}",operators);

            let logs = self.contract_manager.provider.get_logs(&filter).await?;
            info!("logs {:?}",logs);
            for log in logs {
                if let Some(&TxnVerifier::TxnVerificationResult::SIGNATURE_HASH) = log.topic0() {
                    let TxnVerifier::TxnVerificationResult {
                        pubkey,
                        txnHash,
                        blockNumber,
                    } = log.log_decode()?.inner.data;
                    // let task = Task {
                    //     value: taskRequest.value,
                    // };
                    let block_number = log.block_number.expect("Invalid block number");
                    // let task_request = TaskRequest { task, block_number };

                let task = Task {
                    pubkey:pubkey.to_string(),
                    transaction_hash: txnHash.to_string(),
                    block_number:blockNumber.to_string()

                };


                info!("task {:?}",task);


                let task_request = TaskRequest { task, block_number };

                    if !operators.is_empty() {
                        let response = self
                            .send_task_to_all_operators(task_request.task, &operators)
                            .await?;


                            let task_response = TxnVerifier::OperatorResponse {
                                is_included: response.is_included,
                                proposer_index: response.proposer_index.unwrap_or(0) as u64,
                                block_number: response.block_number,
                            };
                            
                            let dss_task_request = TxnVerifier::Task {
                                pubkey:pubkey.to_string(),
                                transaction_hash: txnHash.to_string(),
                                block_number:blockNumber.to_string()
            
        
                            };

                        // let task_response = SquareNumberDSS::TaskResponse { response };
                        // let dss_task_request = SquareNumberDSS::TaskRequest {
                        //     value: task_request.task.value,
                        // };
                        match self
                            .contract_manager
                            .submit_task_response(dss_task_request, task_response)
                            .await
                        {
                            Ok(tx) => info!("Transaction sent: {:?}", tx),
                            Err(e) => error!("Failed to send transaction: {:?}", e),
                        }
                        new_last_checked_block =
                        new_last_checked_block.max(task_request.block_number + 1);
                } else {
                    info!("No operators are registered or no task requests were found.");
                }
            }
        }

        start_block = end_block + 1;
    }

    let _ = self
        .write_block_number_to_file(&self.block_number_store, new_last_checked_block)
        .await;

    Ok(())
}



    // async fn watch_for_task_events(&self) -> Result<()> {
    //     let txn_verifier_address = self.txn_verifier_address;
    //     let next_block_to_check: u64 = self.block_number;
    //     let filter = Filter::new()
    //         .address(txn_verifier_address)
    //         .from_block(BlockNumberOrTag::Number(next_block_to_check));
    
    //     let operators = match self.operator_state.operators.read() {
    //         Ok(guard) => guard.clone(),
    //         Err(PoisonError { .. }) => {
    //             error!("Failed to acquire read lock on operator state");
    //             return Err(eyre::anyhow!(
    //                 "Failed to acquire read lock on operator state"
    //             ));
    //         }
    //     };
        
    //     let logs = self.contract_manager.provider.get_logs(&filter).await?;
    //     let mut new_last_checked_block = next_block_to_check;

    //     info!("logs for mine {:?}",logs);

    //     for log in logs {
    //         if let Some(&TxnVerifier::TxnVerificationResult::SIGNATURE_HASH) = log.topic0() {
    //             let TxnVerifier::TxnVerificationResult {
    //                 pubkey,
    //                 txnHash,
    //                 blockNumber,
    //             } = log.log_decode()?.inner.data;

    //             info!("logs for individial  {:?}",log);
    //             info!("logs_pubkey  {:?}",pubkey);

    //             info!("txnHash_workd   {:?}",txnHash);
    //             info!("blockNumber_world   {:?}",blockNumber);



    //             let task = Task {
    //                 pubkey:pubkey.to_string(),
    //                 transaction_hash: txnHash.to_string(),
    //                 block_number:blockNumber.to_string()

    //             };

    //             info!("operators   {:?}",operators);


    //             if !operators.is_empty() {
    //                 let response = self
    //                     .send_task_to_all_operators(task, &operators)
    //                     .await?;


    //                     info!("response_operators     {:?}",response);



    //                 let task_response = TxnVerifier::OperatorResponse {
    //                     is_included: response.is_included,
    //                     proposer_index: response.proposer_index.unwrap_or(0) as u64,
    //                     block_number: response.block_number,
    //                 };
                    
    //                 let dss_task_request = TxnVerifier::Task {
    //                     pubkey:pubkey.to_string(),
    //                     transaction_hash: txnHash.to_string(),
    //                     block_number:blockNumber.to_string()
    

    //                 };

    //                 // info!("response_operators  {:?}",task_response);

    //                 match self
    //                     .contract_manager
    //                     .submit_task_response(dss_task_request, task_response)
    //                     .await
    //                 {
    //                     Ok(tx) => info!("Transaction sent: {:?}", tx),
    //                     Err(e) => error!("Failed to send transaction: {:?}", e),
    //                 }
    //                 // new_last_checked_block =
    //                     // new_last_checked_block.max(task_request.block_number + 1);
    //             } else {
    //                 info!("No operators are registered or no task requests were found.");
    //             }
    //         }
    //     }
 
 

        
    //     let _ = self
    //         .write_block_number_to_file(&self.block_number_store, new_last_checked_block)
    //         .await;
    
    //     Ok(())
    // }






    async fn get_operator_stake_normalized_eth(
        &self,
        operator: Address,
    ) -> Result<U256, TaskError> {
        let vaults = self
            .contract_manager
            .fetch_vaults_staked_in_dss(operator, self.dss_address)
            .await?;

        let mut stake = Uint::from(0u64);

        for vault in vaults {
            let total_assets =
                VaultContract::new(self.rpc_url.clone(), self.private_key.clone(), vault)?
                    .vault_instance
                    .totalAssets()
                    .call()
                    .await
                    .map_err(|_| TaskError::ContractCallError)?
                    ._0;

            // TODO: Normalize total assets to ETH
            stake += total_assets;
        }

        Ok(stake)
    }

    async fn get_operator_stake_mapping(
        &self,
        operators: Vec<Address>,
        min_acceptable_stake: U256,
    ) -> Result<(HashMap<Address, U256>, U256), TaskError> {
        let mut stake_mapping = HashMap::new();
        let mut total_stake = Uint::from(0u64);

        for operator in operators {
            let stake = self
                .get_operator_stake_normalized_eth(operator)
                .await
                .map_err(TaskError::from)?;

            if stake > min_acceptable_stake {
                stake_mapping.insert(operator, stake);
                total_stake += stake;
            }
        }

        Ok((stake_mapping, total_stake))
    }

    async fn send_task_to_all_operators(
        &self,
        task: Task,
        operators: &HashSet<Operator>,
    ) -> Result<OperatorResponse, TaskError> {
        // Store any error to return if no operator succeeds
        let mut last_error: Option<TaskError> = None;
        info!("send_task_to_all_operators {:?}",task);
    
        for operator in operators.iter() {
            let operator = operator.clone();
    
            match self
                .client
                .post(format!("{}operator/verify", operator.url()))
                .header("Content-Type", "application/json")
                .json(&task)
                .send()
                .await
            {
                Ok(response) => {
                    match response.text().await {
                        Ok(body) => {
                            match serde_json::from_str::<OperatorResponse>(&body) {
                                Ok(operator_response) => {

                                    info!("operator_response {:?}",operator_response);
                                    return Ok(operator_response);
                                }
                                Err(e) => {
                                    error!("Failed to parse operator response: {:?}", e);
                                    // last_error = Some(TaskError::ParseError(e.to_string()));
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to get response body: {:?}", e);
                            // last_error = Some(TaskError::ResponseError(e.to_string()));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get response: {:?}", e);
                    // last_error = Some(TaskError::RequestError(e.to_string()));
                }
            }
        }
    
        // If we got here, no operator succeeded
        Err(last_error.unwrap_or(TaskError::NoOperatorAvailable))
    }

    // async fn verify_message(&self, task_response: &OperatorResponse) -> Result<bool> {
    //     let address: Address = task_response.public_key;
    //     let signature: Signature = task_response.signature;
    //     let message = serde_json::to_string(&task_response.completed_task)?;
    //     let recovered_address = signature.recover_address_from_msg(message)?;
    //     Ok(recovered_address == address)
    // }

    async fn write_block_number_to_file(&self, file: &str, val: u64) -> Result<()> {
        let data = BlockNumberData { block_number: val };

        let json_data = serde_json::to_string_pretty(&data)?;
        fs::write(file, json_data)?;

        Ok(())
    }
}

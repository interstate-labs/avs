use anyhow::Result;
use ethers::{
    prelude::*,
    types::{Address, Filter, Log, U64},
    providers::{Provider, Http},
};
use std::{
    str::FromStr, 
    sync::Arc, 
    time::Duration
};

// Import the ABI from abi.rs
use crate::abi::SymbioticRestaking;

pub struct EventFetcher {
    provider: Arc<Provider<Http>>,
    contract_address: Address,
}

impl EventFetcher {
    pub fn new(rpc_url: &str, contract_address: Address) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        Ok(Self {
            provider: Arc::new(provider),
            contract_address,
        })
    }

    pub async fn start_continuous_fetching(&self,  contract: &SymbioticRestaking<SignerMiddleware<Provider<Http>, LocalWallet>>,  rpc_url: &str) -> Result<()> {
        let mut last_processed_block: U64 = 7799031.into();
        println!("Starting continuous event fetching...");

        loop {
            let latest_block = self.get_latest_block().await?;
            println!("Latest block: {}", latest_block);

            if last_processed_block >= latest_block {
                last_processed_block = 0.into();
            }

            // Fetch events in batches of 900 blocks to stay under 1000-block limit
            let batch_size: u64 = 900;
            let mut current_from_block = last_processed_block.as_u64();
            let mut total_logs = Vec::new();

            while current_from_block < latest_block.as_u64() {
                let current_to_block = std::cmp::min(
                    current_from_block + batch_size, 
                    latest_block.as_u64()
                );

                println!("Fetching events from block {} to {}", current_from_block, current_to_block);
                
                match self.fetch_events(
                    current_from_block.into(), 
                    current_to_block.into()
                ).await {
                    Ok(logs) => {
                        println!("logs {:?}", logs);
                
                        total_logs.extend(logs);
                        current_from_block = current_to_block + 1;
                    }
                    Err(e) => {
                        eprintln!("Error fetching events in batch {}-{}: {}", 
                            current_from_block, current_to_block, e);
                        break;
                    }
                }
            }

            // Process logs
            if !total_logs.is_empty() {
                println!("Found {} total events", total_logs.len());
                self.parse_custom_events(total_logs.clone(),contract,rpc_url)?;
                
                // Update last processed block
                if let Some(last_log_block) = total_logs.last()
                    .and_then(|log| log.block_number) {
                    last_processed_block = last_log_block + 1;
                }
            }

            // Wait before next iteration
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    }

    async fn get_latest_block(&self) -> Result<U64> {
        Ok(self.provider.get_block_number().await?)
    }

    async fn fetch_events(&self, from_block: U64, to_block: U64) -> Result<Vec<Log>> {
        let filter = Filter::new()
            .address(self.contract_address)
            .from_block(from_block)
            .to_block(to_block);

        let logs = self.provider.get_logs(&filter).await?;
        Ok(logs)
    }
    fn parse_custom_events(
        &self, 
        logs: Vec<Log>, 
        contract: &SymbioticRestaking<SignerMiddleware<Provider<Http>, LocalWallet>>,
        rpc_url: &str
    ) -> Result<()> {
        // Specific event signature we're looking for
        let target_event_signature = H256::from_str("0x5a227abd3964ab468a622981e3cafd0f5ee720e97f16d5b249c8be49e366f870").unwrap();
        let rpc_url_clone = rpc_url.to_string();
    
        for log in logs {
            if let Some(topic) = log.topics.get(0) {
                if *topic == target_event_signature && log.topics.len() >= 4 {
                    // Extract details from the topics
                    let validator_id = H256::from_slice(&log.topics[1][0..32]);
                    
                    // Topic 3: Block Number (third topic converted to U256)
                    let block_number = U256::from_big_endian(&log.topics[2][0..32]);
                    
                    // Topic 4: Transaction ID (fourth topic)
                    let tx_id = H256::from_slice(&log.topics[3][0..32]);
    
                    // Block number of the event log
                    let event_block_number = log.block_number.unwrap_or_default();
    
                    // Transaction hash
                    let tx_hash = log.transaction_hash.unwrap_or_default();
    
                    // Clone contract for async use
                    let contract_clone = contract.clone();
                    
                    // Clone RPC URL
                    let local_rpc_url = rpc_url_clone.clone();
    
                    // Spawn an async task
                    tokio::spawn(async move {
                        // Detailed logging of inputs
                        println!("Verification Process Started:");
                        println!("Validator ID: {}", validator_id);
                        println!("Original Block Number: {}", block_number);
                        println!("Transaction ID: {}", tx_id);
    
                        // Create local provider
                        let local_provider = match Provider::<Http>::try_from(local_rpc_url) {
                            Ok(p) => Arc::new(p),
                            Err(e) => {
                                eprintln!("Failed to create provider: {}", e);
                                return;
                            }
                        };
    
                        // Check if the transaction is already verified
                        match contract_clone.get_validator_response(
                            validator_id.to_string(),
                            block_number,
                            tx_id.to_fixed_bytes()
                        ).call().await {
                            Ok(current_verified_status) => {
                              
                              print!("validator_id {:? }",validator_id);
                        
                              print!("tx_id {:? }",tx_id.to_fixed_bytes());

                              print!("block_number {:?}",block_number);


                                println!("Current Verification Status: {}", current_verified_status);
    
                                // If already verified, log and return
                                if current_verified_status {
                                    println!("Transaction already verified at block {}. Skipping.", block_number);
                                    return;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error checking verification status: {}", e);
                                return;
                            }
                        }
    
                        // Check transaction status
                        let status = match Self::check_transaction_in_block(&local_provider, tx_id, block_number).await {
                            Ok(status) => {
                                println!("Transaction Block Check Status: {}", status);
                                status
                            },
                            Err(e) => {
                                eprintln!("Transaction block check failed: {}", e);
                                return;
                            }
                        };
    
                        // Prepare for verified_txn call
                        // let validator_pubkey: Bytes = validator_id.as_bytes().to_vec().into();
                        let tx_id_bytes: [u8; 32] = tx_id.to_fixed_bytes();
    
                        // Generate request hash to debug
                        let request_hash = ethers::utils::keccak256(
                            ethers::abi::encode(&[
                                ethers::abi::Token::Bytes(validator_id.as_bytes().to_vec()),
                                ethers::abi::Token::Uint(block_number),
                                ethers::abi::Token::FixedBytes(tx_id.to_fixed_bytes().to_vec())
                            ])
                        );
                        println!("Generated Request Hash: {}", hex::encode(request_hash));
    
                        // Attempt to call verified_txn
                        match contract_clone.verified_txn(
                            status,  // result
                            validator_id.to_string(),  // validatorPubkey
                            block_number,  // blockNumber
                            tx_id_bytes  // txId
                        ).send().await {
                            Ok(pending_tx) => {
                                println!("verified_txn transaction sent. Hash: {:?}", pending_tx.tx_hash());
                                
                                // Wait for transaction to be mined
                                match pending_tx.await {
                                    Ok(receipt) => {
                                        if let Some(mined_receipt) = receipt {
                                            println!("verified_txn transaction mined in block: {:?}", 
                                                mined_receipt.block_number);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error waiting for verified_txn transaction: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to send verified_txn transaction: {}", e);
                                
                                // Attempt to get more detailed error information
                                if let Some(middleware_error) = e.as_middleware_error() {
                                    println!("Middleware Error Details: {:?}", middleware_error);
                                }
                            }
                        }
    
                        // Print event details
                        println!("Custom Event Detected:");
                        println!("Block Number: {}", event_block_number);
                        println!("Transaction ID: {}", tx_id);
                        println!("Transaction Hash: {}", tx_hash);
                        println!("---");
                    });
                }
            }
        }
        Ok(())
    }
    
    // fn parse_custom_events(
    //     &self, 
    //     logs: Vec<Log>, 
    //     contract: &SymbioticRestaking<SignerMiddleware<Provider<Http>, LocalWallet>>,
    //     rpc_url: &str
    // ) -> Result<()> {
    //     // Specific event signature we're looking for
    //     let target_event_signature = H256::from_str("0x5a227abd3964ab468a622981e3cafd0f5ee720e97f16d5b249c8be49e366f870").unwrap();
    
    //     // Clone the provider for use in the async block
    //     let provider = Arc::clone(&self.provider);
    //     let contract_address = self.contract_address;
    
    //     for log in logs {
    //         // Check if this is the specific event we're interested in
    //         if let Some(topic) = log.topics.get(0) {
    //             if *topic == target_event_signature {
    //                 // Extract details from the topics
    //                 if log.topics.len() >= 4 {
    //                     // Topic 1: Validator ID (second topic)
    //                     let validator_id = H256::from_slice(&log.topics[1][0..32]);
                        
    //                     // Topic 3: Block Number (third topic converted to U256)
    //                     let block_number = U256::from_big_endian(&log.topics[2][0..32]);
                        
    //                     // Topic 4: Transaction ID (fourth topic)
    //                     let tx_id = H256::from_slice(&log.topics[3][0..32]);

    //                     println!("Event Details:");
    //                     println!("Validator ID: {}", hex::encode(validator_id.as_bytes()));
    //                     println!("Block Number: {}", block_number);
    //                     println!("Transaction ID: {}", tx_id);

    
    //                     // Block number of the event log
    //                     let event_block_number = log.block_number.unwrap_or_default();
    
    //                     // Transaction hash
    //                     let tx_hash = log.transaction_hash.unwrap_or_default();
    
    //                     // Clone contract for async use
    //                     let contract_clone = contract.clone();
    //                    let rpc=rpc_url.to_string();
    //                     // Spawn an async task
    //                     tokio::spawn(async move {
    //                         // Create a new provider for this task
    //                         let local_provider = match Provider::<Http>::try_from(rpc) {
    //                             Ok(p) => Arc::new(p),
    //                             Err(e) => {
    //                                 eprintln!("Failed to create provider: {}", e);
    //                                 return;
    //                             }
    //                         };
    
    //                         // Check transaction status
    //                         let status = match Self::check_transaction_in_block(&local_provider, tx_id, block_number).await {
    //                             Ok(status) => status,
    //                             Err(e) => {
    //                                 eprintln!("Transaction check failed: {}", e);
    //                                 return;
    //                             }
    //                         };

                            
    
    //                         // Prepare validator pubkey (convert validator_id to bytes)
    //                         // let validator_pubkey = _validator_id.as_bytes().to_vec();
    //                         // let validator_pubkey: Bytes = _validator_id.as_bytes().to_vec().into();
    //                         let validator_pubkey: Bytes = validator_id.as_bytes().to_vec().into();


    //                         // Call verified_txn on the contract
    //                         let tx_id_bytes: [u8; 32] = tx_id.to_fixed_bytes();
    //                         println!("status tx_id_bytes block_number {:?} , {:?} {:?}", status, block_number,tx_id_bytes);
    //                         match contract_clone.verified_txn(
    //                             status,  // result
    //                             validator_pubkey,  // validatorPubkey
    //                             block_number,  // blockNumber
    //                             tx_id_bytes  // txId
    //                         ).send().await {
    //                             Ok(pending_tx) => {
    //                                 println!("verified_txn transaction sent. Hash: {:?}", pending_tx.tx_hash());
                                    
    //                                 // Optionally wait for the transaction to be mined
    //                                 match pending_tx.await {
    //                                     Ok(receipt) => {
    //                                         println!("verified_txn transaction mined in block: {:?}", 
    //                                             receipt.unwrap().block_number);
    //                                     }
    //                                     Err(e) => {
    //                                         eprintln!("Error waiting for verified_txn transaction: {}", e);
    //                                     }
    //                                 }
    //                             }
    //                             Err(e) => {
    //                                 eprintln!("Failed to send verified_txn transaction: {}", e);
    //                             }
    //                         }
    
    //                         // Print event details
    //                         println!("Custom Event Detected:");
    //                         println!("Block Number: {}", event_block_number);
    //                         println!("Transaction ID: {}", tx_id);
    //                         println!("Transaction Hash: {}", tx_hash);
    //                         println!("---");
    //                     });
    //                 }
    //             }
    //         }
    //     }
    //     Ok(())
    // }
    
    // Separate function for transaction block checking
    async fn check_transaction_in_block(
        provider: &Arc<Provider<Http>>, 
        tx_id: H256, 
        referenced_block: U256
    ) -> Result<bool> {
        // Convert referenced block to U64
        let referenced_block_u64 = referenced_block.as_u64();
    
        // Fetch the transaction receipt
        match provider.get_transaction_receipt(tx_id).await {
            Ok(Some(receipt)) => {
                // Check if the transaction is in the referenced block
                let tx_block_number = receipt.block_number.unwrap_or_default();
                
                println!("Transaction Block Verification:");
                println!("Referenced Block: {}", referenced_block_u64);
                println!("Transaction Block: {}", tx_block_number);
    
                // Compare block numbers
                let is_in_block = tx_block_number.as_u64() == referenced_block_u64;
    
                if is_in_block {
                    println!("✅ Transaction is in the referenced block.");

                } else {
                    println!("❌ Transaction is NOT in the referenced block.");
                }
    
                Ok(is_in_block)
            }
            Ok(None) => {
                println!("❓ Transaction receipt not found.");
                Ok(false)
            }
            Err(e) => {
                eprintln!("Error fetching transaction receipt: {}", e);
                Err(anyhow::anyhow!("Failed to fetch transaction receipt"))
            }
        }
    }
    
}




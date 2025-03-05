// src/abi.rs
use ethers::prelude::*;

abigen!(
    EigenLayerRestaking,
    r#"[
        {
            "inputs": [],
            "name": "UPGRADE_INTERFACE_VERSION",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "avsDirectory",
            "outputs": [{"internalType": "address", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "string",
                    "name": "validatorPubkey",
                    "type": "string"
                },
                {
                    "internalType": "uint256",
                    "name": "blockNumber",
                    "type": "uint256"
                },
                {
                    "internalType": "bytes32",
                    "name": "txId",
                    "type": "bytes32"
                }
            ],
            "name": "slash",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "string",
                    "name": "validatorPubkey",
                    "type": "string"
                },
                {
                    "internalType": "uint256",
                    "name": "blockNumber",
                    "type": "uint256"
                },
                {
                    "internalType": "bytes32",
                    "name": "txId",
                    "type": "bytes32"
                }
            ],
            "name": "get_validator_response",
            "outputs": [
                {
                    "internalType": "bool",
                    "name": "verified",
                    "type": "bool"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "bool",
                    "name": "result",
                    "type": "bool"
                },
                {
                    "internalType": "string",
                    "name": "validatorPubkey",
                    "type": "string"
                },
                {
                    "internalType": "uint256",
                    "name": "blockNumber",
                    "type": "uint256"
                },
                {
                    "internalType": "bytes32",
                    "name": "txId",
                    "type": "bytes32"
                }
            ],
            "name": "verified_txn",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "operator", "type": "address"}],
            "name": "deregisterOperatorFromAVS",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "strategy", "type": "address"}],
            "name": "deregisterStrategy",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getCurrentPeriod",
            "outputs": [{"internalType": "uint48", "name": "periodIndex", "type": "uint48"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "operator", "type": "address"}],
            "name": "getOperatorRestakedStrategies",
            "outputs": [{"internalType": "address[]", "name": "", "type": "address[]"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "operator", "type": "address"},
                {"internalType": "address", "name": "collateral", "type": "address"},
                {"internalType": "uint48", "name": "timestamp", "type": "uint48"}
            ],
            "name": "getOperatorStakeAt",
            "outputs": [{"internalType": "uint256", "name": "amount", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "operator", "type": "address"},
                {"internalType": "address", "name": "collateral", "type": "address"}
            ],
            "name": "getProviderCollateral",
            "outputs": [{"internalType": "uint256", "name": "amount", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "operator", "type": "address"}],
            "name": "getProviderCollateralTokens",
            "outputs": [
                {"internalType": "address[]", "name": "", "type": "address[]"},
                {"internalType": "uint256[]", "name": "", "type": "uint256[]"}
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getRestakeableStrategies",
            "outputs": [{"internalType": "address[]", "name": "", "type": "address[]"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getWhitelistedStrategies",
            "outputs": [{"internalType": "address[]", "name": "", "type": "address[]"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "_owner", "type": "address"},
                {"internalType": "address", "name": "_parameters", "type": "address"},
                {"internalType": "address", "name": "_eigenlayerAVSDirectory", "type": "address"},
                {"internalType": "address", "name": "_eigenlayerDelegationManager", "type": "address"},
                {"internalType": "address", "name": "_eigenlayerStrategyManager", "type": "address"},
                {"internalType": "address", "name": "_restakingHelper", "type": "address"}
            ],
            "name": "initialize",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "strategy", "type": "address"}],
            "name": "isStrategyEnabled",
            "outputs": [{"internalType": "bool", "name": "", "type": "bool"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "owner",
            "outputs": [{"internalType": "address", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "pauseStrategy",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "proxiableUUID",
            "outputs": [{"internalType": "bytes32", "name": "", "type": "bytes32"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "string", "name": "rpc", "type": "string"},
                {"internalType": "string", "name": "rpc1", "type": "string"},
                {"internalType": "string", "name": "rpc2", "type": "string"},
                {
                    "components": [
                        {"internalType": "bytes", "name": "signature", "type": "bytes"},
                        {"internalType": "bytes32", "name": "salt", "type": "bytes32"},
                        {"internalType": "uint256", "name": "expiry", "type": "uint256"}
                    ],
                    "internalType": "struct ISignatureUtils.SignatureWithSaltAndExpiry",
                    "name": "operatorSignature",
                    "type": "tuple"
                }
            ],
            "name": "registerOperator",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "operator", "type": "address"},
                {
                    "components": [
                        {"internalType": "bytes", "name": "signature", "type": "bytes"},
                        {"internalType": "bytes32", "name": "salt", "type": "bytes32"},
                        {"internalType": "uint256", "name": "expiry", "type": "uint256"}
                    ],
                    "internalType": "struct ISignatureUtils.SignatureWithSaltAndExpiry",
                    "name": "operatorSignature",
                    "type": "tuple"
                }
            ],
            "name": "registerOperatorToAVS",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "strategy", "type": "address"}],
            "name": "registerStrategy",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "renounceOwnership",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "restakingHelper",
            "outputs": [{"internalType": "address", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "newOwner", "type": "address"}],
            "name": "transferOwnership",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "unpauseStrategy",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "string", "name": "metadataURI", "type": "string"}],
            "name": "updateAVSMetadataURI",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "newImplementation", "type": "address"},
                {"internalType": "bytes", "name": "data", "type": "bytes"}
            ],
            "name": "upgradeToAndCall",
            "outputs": [],
            "stateMutability": "payable",
            "type": "function"
        }
    ]"#
);

// Re-export the automatically generated SignatureWithSaltAndExpiry type
pub use eigen_layer_restaking::SignatureWithSaltAndExpiry;
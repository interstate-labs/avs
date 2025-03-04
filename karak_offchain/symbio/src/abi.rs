use ethers::prelude::*;

abigen!(
    SymbioticRestaking,
    r#"[
        function getWhitelistedVaults() external view returns (address[] memory)
        function getProviderCollateral(address operator, address collateral) external view returns (uint256 amount)
        function slash(string validatorPubkey, uint256 blockNumber, bytes32 txId) external
        function get_validator_response(string validatorPubkey, uint256 blockNumber, bytes32 txId) external view returns (bool verified)
        function registerOperator(address operatorAddr, string rpc) external
        function isVaultEnabled(address vault) external view returns (bool)
        function getCurrentTime() external view returns (uint48 epoch)
        function initialize(address _owner, address _parameters, address _symbioticNetwork, address _symbioticOperatorRegistry, address _symbioticOperatorNetOptIn, address _symbioticVaultFactory) external
        function verified_txn(bool result, string validatorPubkey, uint256 blockNumber, bytes32 txId) external
    ]"#,
);
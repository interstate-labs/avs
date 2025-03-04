// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;


interface IParameters {
    function VALIDATOR_EPOCH_TIME() external view returns (uint48);
    function PENALTY_WINDOW_DURATION() external view returns (uint48);
    function SKIP_SIGNATURE_VALIDATION() external view returns (bool);
    function CHALLENGE_TIMEOUT_PERIOD() external view returns (uint48);
    function DISPUTE_SECURITY_DEPOSIT() external view returns (uint256);
    function CHAIN_HISTORY_LIMIT() external view returns (uint256);
    function FINALIZATION_DELAY_SLOTS() external view returns (uint256);
    function BEACON_TIME_WINDOW() external view returns (uint256);
    function CONSENSUS_SLOT_DURATION() external view returns (uint256);
    function CONSENSUS_LAUNCH_TIMESTAMP() external view returns (uint256);
    function CONSENSUS_BEACON_ROOT_ADDRESS() external view returns (address);
    function OPERATOR_COLLATERAL_MINIMUM() external view returns (uint256);
}
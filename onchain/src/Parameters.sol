// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";

contract Parameters is OwnableUpgradeable, UUPSUpgradeable {
    // Custom errors
    error InvalidTimeWindow();
    error InvalidPeriod();
    error InvalidAmount();
    error InvalidTimestamp();
    error ZeroAddress();

    // Events
    event SkipSignatureValidationUpdated(bool newValue);
    event ChallengeTimeoutPeriodUpdated(uint48 newPeriod);
    event DisputeSecurityDepositUpdated(uint256 newAmount);
    event OperatorCollateralMinimumUpdated(uint256 newAmount);
    event FinalizationDelaySlotsUpdated(uint256 newValue);

    // Constants and state variables
    uint256 internal constant BEACON_TIME_WINDOW = 8191;
    uint48 public VALIDATOR_EPOCH_TIME;
    uint48 public PENALTY_WINDOW_DURATION;
    bool public SKIP_SIGNATURE_VALIDATION;
    uint48 public CHALLENGE_TIMEOUT_PERIOD;
    uint256 public DISPUTE_SECURITY_DEPOSIT;
    uint256 public CHAIN_HISTORY_LIMIT;
    uint256 public FINALIZATION_DELAY_SLOTS;
    uint256 public CONSENSUS_LAUNCH_TIMESTAMP;
    uint256 public CONSENSUS_SLOT_DURATION;
    uint256 public OPERATOR_COLLATERAL_MINIMUM;
    
    uint256[43] private __gap;

    // Struct to handle initialization parameters
    struct InitializationParams {
        uint48 epochTimeInput;
        uint48 penaltyWindowInput;
        uint48 challengeTimeoutInput;
        bool skipSignatureFlag;
        uint256 securityDepositAmount;
        uint256 historyLimitValue;
        uint256 delaySlotsPeriod;
        uint256 launchTimestampValue;
        uint256 slotDurationValue;
        uint256 minimumCollateralAmount;
    }

    function initialize(
        address adminAddress,
        InitializationParams memory params
    ) public initializer {
        if (adminAddress == address(0)) revert ZeroAddress();
        if (params.epochTimeInput == 0) revert InvalidPeriod();
        if (params.penaltyWindowInput == 0) revert InvalidPeriod();
        if (params.challengeTimeoutInput == 0) revert InvalidPeriod();
        if (params.securityDepositAmount == 0) revert InvalidAmount();
        if (params.minimumCollateralAmount == 0) revert InvalidAmount();
        if (params.launchTimestampValue <= block.timestamp) revert InvalidTimestamp();

        __Ownable_init(adminAddress);

        VALIDATOR_EPOCH_TIME = params.epochTimeInput;
        PENALTY_WINDOW_DURATION = params.penaltyWindowInput;
        SKIP_SIGNATURE_VALIDATION = params.skipSignatureFlag;
        CHALLENGE_TIMEOUT_PERIOD = params.challengeTimeoutInput;
        DISPUTE_SECURITY_DEPOSIT = params.securityDepositAmount;
        CHAIN_HISTORY_LIMIT = params.historyLimitValue;
        FINALIZATION_DELAY_SLOTS = params.delaySlotsPeriod;
        CONSENSUS_LAUNCH_TIMESTAMP = params.launchTimestampValue;
        CONSENSUS_SLOT_DURATION = params.slotDurationValue;
        OPERATOR_COLLATERAL_MINIMUM = params.minimumCollateralAmount;
    }

    // Function to authorize upgrades (required by UUPSUpgradeable)
    function _authorizeUpgrade(
        address implementationContract
    ) internal override onlyOwner {
        if (implementationContract == address(0)) revert ZeroAddress();
    }

    // Setter functions with validation
    function setSkipSignatureValidation(
        bool signatureValidationFlag
    ) public onlyOwner {
        SKIP_SIGNATURE_VALIDATION = signatureValidationFlag;
        emit SkipSignatureValidationUpdated(signatureValidationFlag);
    }

    function setChallengeTimeoutPeriod(
        uint48 timeoutPeriod
    ) public onlyOwner {
        if (timeoutPeriod == 0) revert InvalidPeriod();
        CHALLENGE_TIMEOUT_PERIOD = timeoutPeriod;
        emit ChallengeTimeoutPeriodUpdated(timeoutPeriod);
    }

    function setDisputeSecurityDeposit(
        uint256 securityAmount
    ) public onlyOwner {
        if (securityAmount == 0) revert InvalidAmount();
        DISPUTE_SECURITY_DEPOSIT = securityAmount;
        emit DisputeSecurityDepositUpdated(securityAmount);
    }

    function setOperatorCollateralMinimum(
        uint256 collateralAmount
    ) public onlyOwner {
        if (collateralAmount == 0) revert InvalidAmount();
        OPERATOR_COLLATERAL_MINIMUM = collateralAmount;
        emit OperatorCollateralMinimumUpdated(collateralAmount);
    }

    function setFinalizationDelaySlots(
        uint256 delaySlots
    ) public onlyOwner {
        if (delaySlots == 0) revert InvalidPeriod();
        FINALIZATION_DELAY_SLOTS = delaySlots;
        emit FinalizationDelaySlotsUpdated(delaySlots);
    }


}
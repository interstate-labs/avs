// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

// import {IValidatorRegistrySystem} from "../interfaces/IRegistry.sol";
import {IParameters} from "../interfaces/IParameters.sol";
import {Time} from "@openzeppelin/contracts/utils/types/Time.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {IAVSDirectory} from "@eigenlayer/src/contracts/interfaces/IAVSDirectory.sol";
import {IDelegationManager} from "@eigenlayer/src/contracts/interfaces/IDelegationManager.sol";
import {DelegationManagerStorage} from "@eigenlayer/src/contracts/core/DelegationManagerStorage.sol";
import {IStrategy} from "@eigenlayer/src/contracts/interfaces/IStrategy.sol";
import {StrategyManagerStorage} from "@eigenlayer/src/contracts/core/StrategyManagerStorage.sol";
import {ISignatureUtils} from "@eigenlayer/src/contracts/interfaces/ISignatureUtils.sol";

contract RestakingHelper is OwnableUpgradeable {
    uint48 public START_TIMESTAMP;

    IParameters public parameters;
    // IValidatorRegistrySystem public registry;
    DelegationManagerStorage public DELEGATION_MANAGER;
    IAVSDirectory public AVS_DIRECTORY;

    StrategyManagerStorage public STRATEGY_MANAGER;
    bytes32 public PROTOCOL_IDENTIFIER;

    error NotRegistered();
    error OperationForbidden();

    function initialize(
        address _owner,
        address _parameters,
        address _eigenlayerAVSDirectory,
        address _eigenlayerDelegationManager,
        address _eigenlayerStrategyManager
    ) public initializer {
        __Ownable_init(_owner);
        parameters = IParameters(_parameters);
        // registry = IValidatorRegistrySystem(_registry);
        START_TIMESTAMP = Time.timestamp();

        AVS_DIRECTORY = IAVSDirectory(_eigenlayerAVSDirectory);
        DELEGATION_MANAGER = DelegationManagerStorage(
            _eigenlayerDelegationManager
        );
        STRATEGY_MANAGER = StrategyManagerStorage(_eigenlayerStrategyManager);
        PROTOCOL_IDENTIFIER = keccak256("CONSENSUS_PROTOCOL");
    }

    function _getStartTime() public view returns (uint48) {
        return START_TIMESTAMP;
    }

    function strategy_manager(address strategy) public view returns (bool) {
        // Call strategyIsWhitelistedForDeposit from STRATEGY_MANAGER and return the result
        return
            STRATEGY_MANAGER.strategyIsWhitelistedForDeposit(
                IStrategy(strategy)
            );
    }


    function _checkDelegationIsOperator(
        address Node
    ) public view returns (bool) {
        return DELEGATION_MANAGER.isOperator(Node);
    }


    function _operatorShares(
        address operator,
        IStrategy strategyImpl
    ) public view returns  (uint256) {
        return DELEGATION_MANAGER.operatorShares(operator, strategyImpl);
    }

    function getPeriodAtTime(uint48 periodIndex) public view returns (uint48) {
        return
            (periodIndex - START_TIMESTAMP) / parameters.VALIDATOR_EPOCH_TIME();
    }

    function getPeriodStartTime(
        uint48 epoch
    ) public view returns (uint48 periodIndex) {
        return START_TIMESTAMP + epoch * parameters.VALIDATOR_EPOCH_TIME();
    }

 

    function _avsURI(string calldata metadataURI) public {
        AVS_DIRECTORY.updateAVSMetadataURI(metadataURI);
    }

    function _avsDirector() public view returns (IAVSDirectory)
    {
        return AVS_DIRECTORY;
    }

    function _registerOperatorToAvs(
        address operator,
        ISignatureUtils.SignatureWithSaltAndExpiry memory operatorSignature
    ) public {
        AVS_DIRECTORY.registerOperatorToAVS(operator, operatorSignature);
    }

    function deregisterOperatorFromAVS(address operator) public {
        if (msg.sender != operator) {
            revert OperationForbidden();
        }
        AVS_DIRECTORY.deregisterOperatorFromAVS(operator);
    }
}

// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {EnumerableMap as OEnumerableMap} from "@openzeppelin/contracts/utils/structs/EnumerableMap.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {UUPSUpgradeable} from "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";

import {MapWithTimeData} from "../library/MapWithTimeData.sol";

import {IConsensusRestaking} from "../interfaces/IRestaking.sol";

import {IServiceManager} from "@eigenlayer-middleware/src/interfaces/IServiceManager.sol";
import {IStrategyManager} from "@eigenlayer/src/contracts/interfaces/IStrategyManager.sol";

import {ISignatureUtils} from "@eigenlayer/src/contracts/interfaces/ISignatureUtils.sol";
import {IStrategy} from "@eigenlayer/src/contracts/interfaces/IStrategy.sol";
import {AVSDirectoryStorage} from "@eigenlayer/src/contracts/core/AVSDirectoryStorage.sol";

import "./EigenlayerRestakingHelper.sol";

contract EIgenlayerRestaking is IConsensusRestaking, UUPSUpgradeable, OwnableUpgradeable {
    using OEnumerableMap for OEnumerableMap.AddressToUintMap;
    using MapWithTimeData for OEnumerableMap.AddressToUintMap;

    RestakingHelper public restakingHelper;
    OEnumerableMap.AddressToUintMap private strategies;

    uint256[41] private __gap;

function initialize(
        address _owner,
        address _parameters,
        address _eigenlayerAVSDirectory,
        address _eigenlayerDelegationManager,
        address _eigenlayerStrategyManager,
        address _eigenlayerStrategyManager
    ) public initializer {
        __Ownable_init(_owner);

        // Set the RestakingHelper instance
        restakingHelper = RestakingHelper(_restakingHelper);
        
        // Initialize the RestakingHelper
        RestakingHelper(_restakingHelper).initialize(
            _owner,
            _parameters,
            _eigenlayerAVSDirectory,
            _eigenlayerDelegationManager,
            _eigenlayerStrategyManager
        );
    }


    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    function getCurrentPeriod() public view returns (uint48 periodIndex) {
        return restakingHelper.getPeriodStartTime(Time.timestamp());
    }

    function getWhitelistedStrategies() public view returns (address[] memory) {
        return strategies.keys();
    }

    function registerStrategy(address strategy) public onlyOwner {
        if (strategies.contains(strategy)) {
            revert AlreadyRegistered();
        }

        if (!restakingHelper.strategy_manager(strategy)) {
            revert StrategyNotAllowed();
        }

        strategies.add(strategy);
        strategies.enable(strategy);
    }

    /// @notice Deregister a strategy from working in Bolt Protocol.
    /// @param strategy The EigenLayer strategy address.
    function deregisterStrategy(address strategy) public onlyOwner {
        if (!strategies.contains(strategy)) {
            revert NotRegistered();
        }

        strategies.remove(strategy);
    }
  
  
        function registerOperator(
        string calldata rpc,
         string calldata rpc1,
          string calldata rpc2,
        ISignatureUtils.SignatureWithSaltAndExpiry calldata operatorSignature
    ) public {
   

        if (!restakingHelper._checkDelegationIsOperator(msg.sender)) {
            revert NotOperator();
        }
        registerOperatorToAVS(msg.sender, operatorSignature);


    }

    function pauseStrategy() public {
        if (!strategies.contains(msg.sender)) {
            revert NotRegistered();
        }

        strategies.disable(msg.sender);
    }

    //done
    function unpauseStrategy() public {
        if (!strategies.contains(msg.sender)) {
            revert NotRegistered();
        }

        strategies.enable(msg.sender);
    }

    function isStrategyEnabled(address strategy) public view returns (bool) {
        (uint48 enabledTime, uint48 disabledTime) = strategies.getTimes(
            strategy
        );
        return enabledTime != 0 && disabledTime == 0;
    }

    function getProviderCollateralTokens(
        address operator
    ) public view returns (address[] memory, uint256[] memory) {
        address[] memory collateralTokens = new address[](strategies.length());
        uint256[] memory amounts = new uint256[](strategies.length());

        uint48 epochStartTs = restakingHelper.getPeriodStartTime(
            restakingHelper.getPeriodAtTime(Time.timestamp())
        );

        for (uint256 i = 0; i < strategies.length(); ++i) {
            (
                address strategy,
                uint48 enabledTime,
                uint48 disabledTime
            ) = strategies.atWithTimes(i);

            if (!wasEnabledAt(enabledTime, disabledTime, epochStartTs)) {
                continue;
            }

            IStrategy strategyImpl = IStrategy(strategy);

            address collateral = address(strategyImpl.underlyingToken());
            collateralTokens[i] = collateral;

            uint256 shares = restakingHelper._operatorShares( operator,
                strategyImpl
            );
            amounts[i] = strategyImpl.sharesToUnderlyingView(shares);
        }

        return (collateralTokens, amounts);
    }

    function getProviderCollateral(
        address operator,
        address collateral
    ) public view returns (uint256 amount) {
        uint48 timestamp = Time.timestamp();
        return getOperatorStakeAt(operator, collateral, timestamp);
    }

    function getOperatorStakeAt(
        address operator,
        address collateral,
        uint48 timestamp
    ) public view returns (uint256 amount) {
        if (
            timestamp > Time.timestamp() ||
            timestamp < restakingHelper._getStartTime()
        ) {
            revert InvalidQuery();
        }

        uint48 epochStartTs = restakingHelper.getPeriodStartTime(
            restakingHelper.getPeriodAtTime(timestamp)
        );

        for (uint256 i = 0; i < strategies.length(); i++) {
            (
                address strategy,
                uint48 enabledTime,
                uint48 disabledTime
            ) = strategies.atWithTimes(i);

            if (collateral != address(IStrategy(strategy).underlyingToken())) {
                continue;
            }

            if (!wasEnabledAt(enabledTime, disabledTime, epochStartTs)) {
                continue;
            }

            uint256 shares = restakingHelper._operatorShares(
                operator,
                IStrategy(strategy)
            );
            amount += IStrategy(strategy).sharesToUnderlyingView(shares);
        }

        return amount;
    }

    function updateAVSMetadataURI(
        string calldata metadataURI
    ) public onlyOwner {
        restakingHelper._avsURI(metadataURI);
    }

    function registerOperatorToAVS(
        address operator,
        ISignatureUtils.SignatureWithSaltAndExpiry memory operatorSignature
    ) public {
        restakingHelper._registerOperatorToAvs(
            operator,
            operatorSignature
        );
    }

    function getOperatorRestakedStrategies(
        address operator
    ) external view returns (address[] memory) {
        address[] memory restakedStrategies = new address[](
            strategies.length()
        );

        uint48 epochStartTs = restakingHelper.getPeriodStartTime(
            restakingHelper.getPeriodAtTime(Time.timestamp())
        );

        for (uint256 i = 0; i < strategies.length(); ++i) {
            (
                address strategy,
                uint48 enabledTime,
                uint48 disabledTime
            ) = strategies.atWithTimes(i);

            if (!wasEnabledAt(enabledTime, disabledTime, epochStartTs)) {
                continue;
            }

            if (
                restakingHelper._operatorShares(
                    operator,
                    IStrategy(strategy)
                ) > 0
            ) {
                restakedStrategies[restakedStrategies.length] = strategy;
            }
        }

        return restakedStrategies;
    }

    function wasEnabledAt(
        uint48 enabledTime,
        uint48 disabledTime,
        uint48 timestamp
    ) internal pure returns (bool) {
        return
            enabledTime != 0 &&
            enabledTime <= timestamp &&
            (disabledTime == 0 || disabledTime >= timestamp);
    }

    function getRestakeableStrategies()
        external
        view
        returns (address[] memory)
    {
        return strategies.keys();
    }

    function avsDirectory() external view returns (address) {
        return address(restakingHelper._avsDirector());
    }

 function deregisterOperatorFromAVS(address operator) public {
 
       restakingHelper.deregisterOperatorFromAVS(operator);
 }





 

}

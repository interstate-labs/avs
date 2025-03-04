// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { Test } from "forge-std/Test.sol";
import { ValidatorRegistryCore } from "../src/Registry/ValidatorRegistryCore.sol";
import { IParameters } from "../src/interfaces/IParameters.sol";
import { INodeRegistrationSystem } from "../src/interfaces/IValidators.sol";
import {IValidatorRegistrySystem} from "../src/interfaces/IRegistry.sol";
import { IConsensusRestaking } from "../src/interfaces/IRestaking.sol";


contract MockParameters is IParameters {
    uint48 constant EPOCH_TIME = 1 days;
    uint256 constant MIN_COLLATERAL = 100 ether;
    
    function VALIDATOR_EPOCH_TIME() external pure returns (uint48) {
        return EPOCH_TIME;
    }
    
    function OPERATOR_COLLATERAL_MINIMUM() external pure returns (uint256) {
        return MIN_COLLATERAL;
    }

    function PENALTY_WINDOW_DURATION() external view returns (uint48) { return 0; }
    function SKIP_SIGNATURE_VALIDATION() external view returns (bool) { return false; }
    function CHALLENGE_TIMEOUT_PERIOD() external view returns (uint48) { return 0; }
    function DISPUTE_SECURITY_DEPOSIT() external view returns (uint256) { return 0; }
    function CHAIN_HISTORY_LIMIT() external view returns (uint256) { return 0; }
    function FINALIZATION_DELAY_SLOTS() external view returns (uint256) { return 0; }
    function BEACON_TIME_WINDOW() external view returns (uint256) { return 0; }
    function CONSENSUS_SLOT_DURATION() external view returns (uint256) { return 0; }
    function CONSENSUS_LAUNCH_TIMESTAMP() external view returns (uint256) { return 0; }
    function CONSENSUS_BEACON_ROOT_ADDRESS() external view returns (address) { return address(0); }
}

contract MockNodeRegistration is INodeRegistrationSystem {
    mapping(bytes20 => ValidatorNodeDetails) public nodes;

    function setNode(bytes20 identityHash, ValidatorNodeDetails memory details) external {
        nodes[identityHash] = details;
    }

    function fetchNodeByIdentityHash(bytes20 identityHash) external view returns (ValidatorNodeDetails memory) {
        return nodes[identityHash];
    }
}

contract MockConsensusRestaking is IConsensusRestaking {
    mapping(address => mapping(address => uint256)) private collaterals;
    mapping(address => address[]) private collateralTokens;
    mapping(address => uint256[]) private collateralAmounts;

    function setProviderCollateral(address provider, address token, uint256 amount) external {
        collaterals[provider][token] = amount;
        
        // Update tokens and amounts arrays
        bool exists = false;
        for (uint i = 0; i < collateralTokens[provider].length; i++) {
            if (collateralTokens[provider][i] == token) {
                collateralAmounts[provider][i] = amount;
                exists = true;
                break;
            }
        }
        if (!exists) {
            collateralTokens[provider].push(token);
            collateralAmounts[provider].push(amount);
        }
    }

    function getProviderCollateral(address provider, address token) external view returns (uint256) {
        return collaterals[provider][token];
    }

    function getProviderCollateralTokens(address provider) external view returns (address[] memory, uint256[] memory) {
        return (collateralTokens[provider], collateralAmounts[provider]);
    }
}

contract ValidatorRegistryTest is Test {
    ValidatorRegistryCore public registry;
    MockParameters public parameters;
    MockNodeRegistration public nodeRegistration;
    MockConsensusRestaking public consensusRestaking;
    
    address public admin = address(1);
    address public protocol1 = address(2);
    address public protocol2 = address(3);
    address public validator1 = address(4);
    address public validator2 = address(5);
    bytes20 public validatorHash1 = bytes20(uint160(1000));
    bytes20 public validatorHash2 = bytes20(uint160(2000));
    address public collateralToken1 = address(6);
    address public collateralToken2 = address(7);
    
    function setUp() public {
        // Deploy mock contracts
        parameters = new MockParameters();
        nodeRegistration = new MockNodeRegistration();
        consensusRestaking = new MockConsensusRestaking();
        
        // Deploy and initialize registry
        registry = new ValidatorRegistryCore();
        registry.initializeSystem(
            admin,
            address(parameters),
            address(nodeRegistration)
        );
        
        vm.startPrank(admin);
        registry.registerProtocol(protocol1);
        registry.registerProtocol(protocol2);
        vm.stopPrank();

        // Setup mock validator nodes
        nodeRegistration.setNode(validatorHash1, INodeRegistrationSystem.ValidatorNodeDetails({
            nodeIdentityHash: validatorHash1,
            gasCapacityLimit: 1000000,
            assignedOperatorAddress: validator1,
            controllerAddress: address(10)
        }));
    }

    function testInitialization() public {
        assertEq(registry.owner(), admin);
        assertTrue(registry.SYSTEM_INITIALIZATION_TIME() > 0);
    }

    function testProtocolManagement() public {
        vm.startPrank(admin);
        
        // Test registration
        address newProtocol = address(100);
        registry.registerProtocol(newProtocol);
        address[] memory protocols = registry.listSupportedProtocols();
        assertTrue(protocols.length == 3);
        
        // Test deregistration
        registry.deregisterProtocol(newProtocol);
        protocols = registry.listSupportedProtocols();
        assertTrue(protocols.length == 2);
        
        vm.stopPrank();
    }

    // function testValidatorNodeOperations() public {
    //     vm.startPrank(protocol1);
        
    //     // Test enrollment
    //     string memory endpoint = "https://validator1.example.com";
    //     registry.enrollValidatorNode(validator1, endpoint);
    //     assertTrue(registry.validateNodeRegistration(validator1));
        
    //     // Test suspension
    //     registry.suspendValidatorNode(validator1);
    //     assertFalse(registry.checkNodeOperationalStatus(validator1));
        
    //     // Test reactivation
    //     registry.reactivateValidatorNode(validator1);
    //     assertTrue(registry.checkNodeOperationalStatus(validator1));
        
    //     // Test removal
    //     registry.removeValidatorNode(validator1);
    //     assertFalse(registry.validateNodeRegistration(validator1));
        
    //     vm.stopPrank();
    // }

    function testValidatorProfileFetching() public {
        // Setup collateral for validator
        consensusRestaking.setProviderCollateral(validator1, collateralToken1, 150 ether);
        consensusRestaking.setProviderCollateral(validator1, collateralToken2, 50 ether);

        vm.startPrank(protocol1);
        registry.enrollValidatorNode(validator1, "https://validator1.example.com");
        
        // Test single profile fetch
        IValidatorRegistrySystem.ValidatorNodeProfile memory profile = registry.fetchValidatorProfile(validatorHash1);
        assertEq(profile.validatorIdentityHash, validatorHash1);
        assertEq(profile.nodeManagerAddress, validator1);
        
        // Test batch profile fetch
        bytes20[] memory hashes = new bytes20[](1);
        hashes[0] = validatorHash1;
        IValidatorRegistrySystem.ValidatorNodeProfile[] memory profiles = registry.fetchValidatorProfileBatch(hashes);
        assertEq(profiles.length, 1);
        assertEq(profiles[0].validatorIdentityHash, validatorHash1);
        
        vm.stopPrank();
    }

    // function testCollateralCalculations() public {
    //     vm.startPrank(protocol1);
    //     registry.enrollValidatorNode(validator1, "https://validator1.example.com");
        
    //     // Setup collateral
    //     consensusRestaking.setProviderCollateral(validator1, collateralToken1, 150 ether);
    //     consensusRestaking.setProviderCollateral(validator1, collateralToken2, 50 ether);
        
    //     // Test individual collateral fetch
    //     uint256 collateral = registry.fetchNodeCollateralAmount(validator1, collateralToken1);
    //     assertEq(collateral, 150 ether);
        
    //     // Test total collateral calculation
    //     uint256 totalCollateral = registry.calculateTotalCollateral(collateralToken1);
    //     assertEq(totalCollateral, 150 ether);
        
    //     vm.stopPrank();
    // }

    function testEpochTimeCalculations() public {
        uint48 currentEpoch = registry.fetchCurrentEpoch();
        uint48 startTime = registry.calculateEpochStartTime(currentEpoch);
        
        assertTrue(startTime <= uint48(block.timestamp));
        assertTrue(uint48(block.timestamp) < startTime + uint48(parameters.VALIDATOR_EPOCH_TIME()));
    }

    function testValidatorAuthorization() public {
        bool isAuthorized = registry.validateNodeAuthorization(validator1, validatorHash1);
        assertTrue(isAuthorized);
        
        // Test with invalid hash
        vm.expectRevert();
        registry.validateNodeAuthorization(validator1, bytes20(0));
    }

    // function testFailureCases() public {
    //     // Test unauthorized protocol access
    //     vm.startPrank(address(999));
    //     vm.expectRevert();
    //     registry.enrollValidatorNode(validator1, "https://validator1.example.com");
    //     vm.stopPrank();
        
    //     // Test duplicate enrollment
    //     vm.startPrank(protocol1);
    //     registry.enrollValidatorNode(validator2, "https://validator2.example.com");
    //     vm.expectRevert();
    //     registry.enrollValidatorNode(validator2, "https://validator2-new.example.com");
    //     vm.stopPrank();
        
    //     // Test operations on non-existent validator
    //     vm.startPrank(protocol1);
    //     vm.expectRevert();
    //     registry.suspendValidatorNode(address(999));
    //     vm.stopPrank();
    // }

    // function testUpgradeability() public {
    //     vm.startPrank(admin);
    //     // Test upgrade authorization
    //     registry._authorizeUpgrade(address(100));
    //     vm.stopPrank();

    //     // Test unauthorized upgrade
    //     vm.startPrank(address(999));
    //     vm.expectRevert();
    //     registry._authorizeUpgrade(address(100));
    //     vm.stopPrank();
    // }
}

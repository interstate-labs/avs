// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import {ValidatorsLib} from "./library/ValidatorsLib.sol";
import {IParameters} from "./interfaces/IParameters.sol";
import {INodeRegistrationSystem} from "./interfaces/IValidators.sol";
import {BLS12381} from "./library/bls/BLS12381.sol";


contract Validator is
    OwnableUpgradeable,
    UUPSUpgradeable,
    INodeRegistrationSystem
{
    using ValidatorsLib for ValidatorsLib.ValidatorSet;
    using BLS12381 for BLS12381.G1Point;
    using ValidatorsLib for ValidatorsLib.ValidatorSet;
    ValidatorsLib.ValidatorSet internal NODES;
    IParameters public protocolParameters;

    error UnauthorizedAccessAttempt();
    event ConsensusNodeRegistered(bytes32 indexed nodeIdentityHash);

    uint256[43] private __gap;

    function initialize(address _owner, address _parameters)
        public
        initializer
    {
        __Ownable_init(_owner);
        protocolParameters = IParameters(_parameters);
    }

    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}

    function fetchAllValidatorNodes()
        public
        view
        returns (ValidatorNodeDetails[] memory)
    {
        ValidatorsLib._Validator[] memory _nodes = NODES.getAll();
        ValidatorNodeDetails[] memory nodes = new ValidatorNodeDetails[](
            _nodes.length
        );
        for (uint256 i = 0; i < _nodes.length; i++) {
            nodes[i] = _getNodeInfo(_nodes[i]);
        }
        return nodes;
    }

    function updateNodeCapacity(
        bytes20 nodeIdentityHash,
        uint32 maxGasCommitment
    ) public {
        address controller = NODES.getController(nodeIdentityHash);
        if (msg.sender != controller) {
            revert UnauthorizedAccessAttempt();
        }

        NODES.updateMaxCommittedGasLimit(nodeIdentityHash, maxGasCommitment);
    }

    function fetchValidatorByIdentityHash(bytes20 nodeIdentityHash)
        public
        view
        returns (ValidatorNodeDetails memory)
    {
        ValidatorsLib._Validator memory _node = NODES.get(nodeIdentityHash);
        return _getNodeInfo(_node);
    }

    function _getNodeInfo(ValidatorsLib._Validator memory _node)
        internal
        view
        returns (INodeRegistrationSystem.ValidatorNodeDetails memory)
    {
        return
            INodeRegistrationSystem.ValidatorNodeDetails({
                pubkey: _node.pubkey,
                rpcs: _node.rpcs,
                nodeIdentityHash: _node.pubkeyHash,
                gasCapacityLimit: _node.maxCommittedGasLimit,
             
                controllerAddress: NODES.getController(_node.pubkeyHash)
            });
    }

    function _registerNode(
        BLS12381.G1Point memory pubkey,
        string memory rpc,
        bytes20 nodeIdentityHash,
        address operatorAddress,
        uint32 maxGasCommitment
    ) internal {
        if (operatorAddress == address(0)) {
            revert INodeRegistrationSystem.InvalidOperatorAssignment();
        }
        if (nodeIdentityHash == bytes20(0)) {
            revert INodeRegistrationSystem.InvalidNodeIdentity();
        }

        NODES.insert(
            pubkey,
            rpc,
            nodeIdentityHash,
            maxGasCommitment,
            NODES.getOrInsertController(msg.sender)
        );
        emit ConsensusNodeRegistered(nodeIdentityHash);
    }

    function _batchRegisterNodes(
        BLS12381.G1Point[] memory pubkeys,
        string memory rpcs,
        bytes20[] memory keyHashes,
        uint32 maxGasCommitment
    ) internal {
     
   
        uint32 controllerIndex = NODES.getOrInsertController(msg.sender);

        for (uint32 i; i < keyHashes.length; i++) {
            bytes20 nodeIdentityHash = keyHashes[i];
            if (nodeIdentityHash == bytes20(0)) {
                revert INodeRegistrationSystem.InvalidNodeIdentity();
            }


            NODES.insert(
                pubkeys[i],
                rpcs,
                nodeIdentityHash,
                maxGasCommitment,
                controllerIndex
            );
            emit ConsensusNodeRegistered(nodeIdentityHash);
        }
    }

    function fetchValidatorByPublicKey(BLS12381.G1Point calldata pubkey)
        public
        view
        returns (ValidatorNodeDetails memory)
    {
        return fetchValidatorByIdentityHash(computeNodeIdentityHash(pubkey));
    }

 

    function enrollValidatorWithVerification(
        BLS12381.G1Point calldata pubkey,
        string calldata rpc,
        uint32 maxGasCommitment,
        address operatorAddress
    ) public {
        _registerNode(
            pubkey,
            rpc,
            computeNodeIdentityHash(pubkey),
            operatorAddress,
            maxGasCommitment
        );
    }

    function bulkEnrollValidatorsWithVerification(
        BLS12381.G1Point[] memory pubkeys,
        string memory rpcs,
        uint32 maxGasCommitment
    ) public {

        bytes20[] memory keyHashes = new bytes20[](pubkeys.length);
        for (uint256 i = 0; i < pubkeys.length; i++) {
            keyHashes[i] = computeNodeIdentityHash(pubkeys[i]);
        }

        _batchRegisterNodes(
            pubkeys,
            rpcs,
            keyHashes,
            maxGasCommitment
        );
    }


    function computeNodeIdentityHash(BLS12381.G1Point memory pubkey)
        public
        pure
        returns (bytes20)
    {
        uint256[2] memory compressed = pubkey.compress();
        bytes32 fullHash = keccak256(abi.encodePacked(compressed));
        return bytes20(uint160(uint256(fullHash)));
    }
}

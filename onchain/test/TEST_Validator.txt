// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;


import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";

import {BLS12381} from "./lib/bls/BLS12381.sol";
import {BLSSignatureVerifier} from "./lib/bls/BLSSignatureVerifier.sol";
import {ValidatorsLib} from "./lib/ValidatorsLib.sol";

import {INodeRegistrationSystem} from "./interfaces/IValidators.sol";
import {IParameters} from "./interfaces/IParameters.sol";


contract ConsensusNodeRegistry is
    INodeRegistrationSystem,
    BLSSignatureVerifier,
    OwnableUpgradeable,
    UUPSUpgradeable
{
    using BLS12381 for BLS12381.G1Point;
    using ValidatorsLib for ValidatorsLib.ValidatorSet;

    IParameters public protocolParameters;

    ValidatorsLib.ValidatorSet internal NODES;

    uint256[43] private __gap;

    event ConsensusNodeRegistered(bytes32 indexed nodeIdentityHash);

    function initialize(
        address _owner,
        address _parameters
    ) public initializer {
        __Ownable_init(_owner);
        protocolParameters = IParameters(_parameters);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

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

    function fetchNodeByPublicKey(
        BLS12381.G1Point calldata pubkey
    ) public view returns (ValidatorNodeDetails memory) {
        return fetchNodeByIdentityHash(computeNodeIdentityHash(pubkey));
    }

    function fetchNodeByIdentityHash(
        bytes20 nodeIdentityHash
    ) public view returns (ValidatorNodeDetails memory) {
        ValidatorsLib._Validator memory _node = NODES.get(nodeIdentityHash);
        return _getNodeInfo(_node);
    }

    function enrollNodeWithoutVerification(
        bytes20 nodeIdentityHash,
        uint32 maxGasCommitment,
        address operatorAddress
    ) public {
        if (!protocolParameters.SKIP_SIGNATURE_VALIDATION()) {
            revert SecureRegistrationRequired();
        }

        _registerNode(nodeIdentityHash, operatorAddress, maxGasCommitment);
    }

    function enrollNodeWithVerification(
        BLS12381.G1Point calldata pubkey,
        BLS12381.G2Point calldata signature,
        uint32 maxGasCommitment,
        address operatorAddress
    ) public {
        uint32 sequenceNumber = uint32(NODES.length() + 1);
        bytes memory message = abi.encodePacked(
            block.chainid,
            msg.sender,
            sequenceNumber
        );
        if (!_verifySignature(message, signature, pubkey)) {
            revert SignatureVerificationFailed();
        }

        _registerNode(
            computeNodeIdentityHash(pubkey),
            operatorAddress,
            maxGasCommitment
        );
    }

    function bulkEnrollNodesWithVerification(
        BLS12381.G1Point[] calldata pubkeys,
        BLS12381.G2Point calldata signature,
        uint32 maxGasCommitment,
        address operatorAddress
    ) public {
        uint32[] memory sequenceNumbers = new uint32[](pubkeys.length);
        uint32 nextSequenceNumber = uint32(NODES.length() + 1);
        for (uint32 i = 0; i < pubkeys.length; i++) {
            sequenceNumbers[i] = nextSequenceNumber + i;
        }

        bytes memory message = abi.encodePacked(
            block.chainid,
            msg.sender,
            sequenceNumbers
        );
        BLS12381.G1Point memory aggregatedPubkey = _aggregatePubkeys(pubkeys);

        if (!_verifySignature(message, signature, aggregatedPubkey)) {
            revert SignatureVerificationFailed();
        }

        bytes20[] memory keyHashes = new bytes20[](pubkeys.length);
        for (uint256 i = 0; i < pubkeys.length; i++) {
            keyHashes[i] = computeNodeIdentityHash(pubkeys[i]);
        }

        _batchRegisterNodes(keyHashes, operatorAddress, maxGasCommitment);
    }

    function bulkEnrollNodesWithoutVerification(
        bytes20[] calldata keyHashes,
        uint32 maxGasCommitment,
        address operatorAddress
    ) public {
        if (!protocolParameters.SKIP_SIGNATURE_VALIDATION()) {
            revert SecureRegistrationRequired();
        }

        _batchRegisterNodes(keyHashes, operatorAddress, maxGasCommitment);
    }

    function updateNodeCapacity(
        bytes20 nodeIdentityHash,
        uint32 maxGasCommitment
    ) public {
        address controller = NODES.getController(nodeIdentityHash);
        if (msg.sender != controller) {
            // revert UnauthorizedAccessAttempt();
        }

        NODES.updateMaxCommittedGasLimit(nodeIdentityHash, maxGasCommitment);
    }

    function _registerNode(
        bytes20 nodeIdentityHash,
        address operatorAddress,
        uint32 maxGasCommitment
    ) internal {
        if (operatorAddress == address(0)) {
            revert InvalidOperatorAssignment();
        }
        if (nodeIdentityHash == bytes20(0)) {
            revert InvalidNodeIdentity();
        }

        NODES.insert(
            nodeIdentityHash,
            maxGasCommitment,
            NODES.getOrInsertController(msg.sender),
            NODES.getOrInsertAuthorizedOperator(operatorAddress)
        );
        emit ConsensusNodeRegistered(nodeIdentityHash);
    }

    function _batchRegisterNodes(
        bytes20[] memory keyHashes,
        address operatorAddress,
        uint32 maxGasCommitment
    ) internal {
        if (operatorAddress == address(0)) {
            revert InvalidOperatorAssignment();
        }

        uint32 operatorIndex = NODES.getOrInsertAuthorizedOperator(
            operatorAddress
        );
        uint32 controllerIndex = NODES.getOrInsertController(msg.sender);

        for (uint32 i; i < keyHashes.length; i++) {
            bytes20 nodeIdentityHash = keyHashes[i];
            if (nodeIdentityHash == bytes20(0)) {
                revert InvalidNodeIdentity();
            }

            NODES.insert(
                nodeIdentityHash,
                maxGasCommitment,
                controllerIndex,
                operatorIndex
            );
            emit ConsensusNodeRegistered(nodeIdentityHash);
        }
    }

    function _getNodeInfo(
        ValidatorsLib._Validator memory _node
    ) internal view returns (ValidatorNodeDetails memory) {
        return
            ValidatorNodeDetails({
                nodeIdentityHash: _node.pubkeyHash,
                gasCapacityLimit: _node.maxCommittedGasLimit,
                assignedOperatorAddress: NODES.getAuthorizedOperator(
                    _node.pubkeyHash
                ),
                controllerAddress: NODES.getController(_node.pubkeyHash)
            });
    }

    function computeNodeIdentityHash(
        BLS12381.G1Point memory pubkey
    ) public pure returns (bytes20) {
        uint256[2] memory compressed = pubkey.compress();
        bytes32 fullHash = keccak256(abi.encodePacked(compressed));
        return bytes20(uint160(uint256(fullHash)));
    }

 
}
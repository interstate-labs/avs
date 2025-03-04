// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import {Time} from "@openzeppelin/contracts/utils/types/Time.sol";

import {RLPReader} from "./library/rlp/RLPReader.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import {TransactionDecoder} from "./library/TransactionDecoder.sol";

import {IParameters} from "./interfaces/IParameters.sol";
import {ISlashing} from "./interfaces/ISlashing.sol";
import {RLPWriter} from "./library/rlp/RLPWriter.sol";
import {MerkleTrie} from "./library/trie/MerkleTrie.sol";
import {SecureMerkleTrie} from "./library/trie/SecureMerkleTrie.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract ValidationProcessor is OwnableUpgradeable, UUPSUpgradeable, ISlashing {
    using EnumerableSet for EnumerableSet.Bytes32Set;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;
    using TransactionDecoder for bytes;
    using TransactionDecoder for TransactionDecoder.Transaction;
    EnumerableSet.Bytes32Set internal validationSetIDs;
    mapping(bytes32 => ValidationRecord) internal validationRecords;
    IParameters public validatorParams;

    using EnumerableSet for EnumerableSet.Bytes32Set;
    using TransactionDecoder for TransactionDecoder.Transaction;
    uint256[46] private __gap;

    function initialize(
        address _owner,
        address _parameters
    ) public initializer {
        __Ownable_init(_owner);
        validatorParams = IParameters(_parameters);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    function concludeAwaitingValidation(
        bytes32 validationId,
        ValidationEvidence calldata evidence
    ) public {
        if (
            validationRecords[validationId].targetEpoch <
            _getCurrentEpoch() - validatorParams.CHAIN_HISTORY_LIMIT()
        ) {
            revert SegmentTooAgedError();
        }

        uint256 previousSegmentHeight = evidence.incorporationHeight - 1;
        if (
            previousSegmentHeight > block.number ||
            previousSegmentHeight <
            block.number - validatorParams.CHAIN_HISTORY_LIMIT()
        ) {
            revert InvalidSegmentHeightError();
        }

        bytes32 trustedPreviousSegmentHash = blockhash(
            evidence.incorporationHeight
        );
        verifyAndFinalize(validationId, trustedPreviousSegmentHash, evidence);
    }

    function _getTimestampFromEpoch(
        uint256 _epoch
    ) internal view returns (uint256) {
        return
            validatorParams.CONSENSUS_LAUNCH_TIMESTAMP() +
            _epoch *
            validatorParams.VALIDATOR_EPOCH_TIME();
    }

    function _getConsensusRootAt(
        uint256 _epoch
    ) internal view returns (bytes32) {
        uint256 slotTimestamp = validatorParams.CONSENSUS_LAUNCH_TIMESTAMP() +
            _epoch *
            validatorParams.VALIDATOR_EPOCH_TIME();
        return _getConsensusRootFromTimestamp(slotTimestamp);
    }

    function _getConsensusRootFromTimestamp(
        uint256 _timestamp
    ) internal view returns (bytes32) {
        (bool success, bytes memory data) = validatorParams
            .CONSENSUS_BEACON_ROOT_ADDRESS()
            .staticcall(abi.encode(_timestamp));

        if (!success || data.length == 0) {
            revert ConsensusRootMissingError();
        }

        return abi.decode(data, (bytes32));
    }

    function _getLatestBeaconBlockRoot() internal view returns (bytes32) {
        uint256 latestSlot = _getEpochFromTimestamp(block.timestamp);
        return _getConsensusRootAt(latestSlot);
    }

    function _isWithinEIP4788Window(
        uint256 _timestamp
    ) internal view returns (bool) {
        return
            _getEpochFromTimestamp(_timestamp) <=
            _getCurrentEpoch() + validatorParams.BEACON_TIME_WINDOW();
    }

    function recoverAuthorizationData(
        AuthorizedMessagePacket calldata authorization
    )
        internal
        pure
        returns (
            address msgSender,
            address witnessAuthorizer,
            MessageDetails memory messageData
        )
    {
        witnessAuthorizer = ECDSA.recover(
            computeAuthorizationId(authorization),
            authorization.authorization
        );
        TransactionDecoder.Transaction memory decodedMsg = authorization
            .payload
            .decodeEnveloped();
        msgSender = decodedMsg.recoverSender();
        messageData = MessageDetails({
            messageDigest: keccak256(authorization.payload),
            sequence: decodedMsg.nonce,
            fuelLimit: decodedMsg.gasLimit
        });
    }

    function computeValidationId(
        AuthorizedMessagePacket[] calldata authorizations
    ) internal pure returns (bytes32) {
        bytes32[] memory signatures = new bytes32[](authorizations.length);
        for (uint256 i = 0; i < authorizations.length; i++) {
            signatures[i] = keccak256(authorizations[i].authorization);
        }
        return keccak256(abi.encodePacked(signatures));
    }

    function computeAuthorizationId(
        AuthorizedMessagePacket calldata authorization
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    keccak256(authorization.payload),
                    toLittleEndian(authorization.epoch)
                )
            );
    }

    function toLittleEndian(uint64 x) internal pure returns (bytes memory) {
        bytes memory b = new bytes(8);
        for (uint256 i = 0; i < 8; i++) {
            b[i] = bytes1(uint8(x >> (8 * i)));
        }
        return b;
    }

    function _decodeSegmentHeaderRLP(
        bytes calldata headerRLP
    ) internal pure returns (ChainSegmentInfo memory segmentInfo) {
        RLPReader.RLPItem[] memory headerFields = headerRLP
            .toRLPItem()
            .readList();

        segmentInfo.ancestorDigest = headerFields[0].readBytes32();
        segmentInfo.worldStateDigest = headerFields[3].readBytes32();
        segmentInfo.messageTreeDigest = headerFields[4].readBytes32();
        segmentInfo.segmentHeight = headerFields[8].readUint256();
        segmentInfo.chronograph = headerFields[11].readUint256();
        segmentInfo.networkFee = headerFields[15].readUint256();
    }

    function _finalizeValidation(
        ValidationPhase outcome,
        ValidationRecord storage record
    ) internal {
        if (outcome == ValidationPhase.Confirmed) {
            record.phase = ValidationPhase.Confirmed;
            _distributeHalfDeposit(msg.sender);
            _distributeHalfDeposit(record.witnessAuthorizer);
            emit ValidationConfirmed(record.attestationId);
        } else if (outcome == ValidationPhase.Rejected) {
            record.phase = ValidationPhase.Rejected;
            _distributeFullDeposit(record.validator);
            emit ValidationRejected(record.attestationId);
        }

        delete validationRecords[record.attestationId];
        validationSetIDs.remove(record.attestationId);
    }

    function _distributeHalfDeposit(address recipient) internal {
        (bool success, ) = payable(recipient).call{
            value: validatorParams.DISPUTE_SECURITY_DEPOSIT() / 2
        }("");
        if (!success) {
            revert BondTransferFailedError();
        }
    }

    function _distributeFullDeposit(address recipient) internal {
        (bool success, ) = payable(recipient).call{
            value: validatorParams.DISPUTE_SECURITY_DEPOSIT()
        }("");
        if (!success) {
            revert BondTransferFailedError();
        }
    }

    function _getEpochFromTimestamp(
        uint256 _timestamp
    ) internal view returns (uint256) {
        return
            (_timestamp - validatorParams.CONSENSUS_LAUNCH_TIMESTAMP()) /
            validatorParams.VALIDATOR_EPOCH_TIME();
    }

    function _getCurrentEpoch() internal view returns (uint256) {
        return _getEpochFromTimestamp(block.timestamp);
    }

    function verifyAndFinalize(
        bytes32 validationId,
        bytes32 trustedPreviousSegmentHash,
        ValidationEvidence calldata evidence
    ) internal {
        if (!validationSetIDs.contains(validationId)) {
            revert ValidationNotFoundError();
        }

        ValidationRecord storage record = validationRecords[validationId];

        if (record.phase != ValidationPhase.Awaiting) {
            revert ValidationAlreadySettledError();
        }

        if (
            record.timestampInit + validatorParams.CHALLENGE_TIMEOUT_PERIOD() <
            Time.timestamp()
        ) {
            revert ValidationTimedOutError();
        }

        uint256 messageCount = record.authorizedMessages.length;
        if (
            evidence.messageMerkleEvidence.length != messageCount ||
            evidence.messagePositions.length != messageCount
        ) {
            revert InvalidEvidenceCountError();
        }

        bytes32 previousSegmentHash = keccak256(evidence.precedingSegmentRLP);
        if (previousSegmentHash != trustedPreviousSegmentHash) {
            revert InvalidSegmentDigestError();
        }

        ChainSegmentInfo memory previousSegment = _decodeSegmentHeaderRLP(
            evidence.precedingSegmentRLP
        );
        ChainSegmentInfo memory incorporationSegment = _decodeSegmentHeaderRLP(
            evidence.incorporationSegmentRLP
        );

        if (incorporationSegment.ancestorDigest != previousSegmentHash) {
            revert InvalidAncestorDigestError();
        }

        (bool participantExists, bytes memory participantRLP) = SecureMerkleTrie
            .get(
                abi.encodePacked(record.protocolDestination),
                evidence.participantMerkleEvidence,
                previousSegment.worldStateDigest
            );

        if (!participantExists) {
            revert ParticipantNotFoundError();
        }

        ParticipantState memory participant = _decodeParticipantRLP(
            participantRLP
        );

        for (uint256 i = 0; i < messageCount; i++) {
            MessageDetails memory message = record.authorizedMessages[i];

            if (participant.sequence > message.sequence) {
                _finalizeValidation(ValidationPhase.Confirmed, record);
                return;
            }

            if (
                participant.holdings <
                incorporationSegment.networkFee * message.fuelLimit
            ) {
                _finalizeValidation(ValidationPhase.Confirmed, record);
                return;
            }

            participant.holdings -=
                incorporationSegment.networkFee *
                message.fuelLimit;
            participant.sequence++;

            bytes memory messageLeaf = RLPWriter.writeUint(
                evidence.messagePositions[i]
            );

            (bool messageExists, bytes memory messageRLP) = MerkleTrie.get(
                messageLeaf,
                evidence.messageMerkleEvidence[i],
                incorporationSegment.messageTreeDigest
            );

            if (!messageExists) {
                revert MessageNotFoundError();
            }

            if (message.messageDigest != keccak256(messageRLP)) {
                revert InvalidMessageEvidenceError();
            }
        }

        _finalizeValidation(ValidationPhase.Confirmed, record);
    }

    function _decodeParticipantRLP(
        bytes memory participantRLP
    ) internal pure returns (ParticipantState memory participant) {
        RLPReader.RLPItem[] memory participantFields = participantRLP
            .toRLPItem()
            .readList();
        participant.sequence = participantFields[0].readUint256();
        participant.holdings = participantFields[1].readUint256();
    }

    function retrieveAllValidations()
        public
        view
        returns (ValidationRecord[] memory)
    {
        ValidationRecord[] memory allValidations = new ValidationRecord[](
            validationSetIDs.length()
        );
        for (uint256 i = 0; i < validationSetIDs.length(); i++) {
            allValidations[i] = validationRecords[validationSetIDs.at(i)];
        }
        return allValidations;
    }

    function retrieveAwaitingValidations()
        public
        view
        returns (ValidationRecord[] memory)
    {
        uint256 pendingCount = 0;
        for (uint256 i = 0; i < validationSetIDs.length(); i++) {
            if (
                validationRecords[validationSetIDs.at(i)].phase ==
                ValidationPhase.Awaiting
            ) {
                pendingCount++;
            }
        }

        ValidationRecord[] memory pendingValidations = new ValidationRecord[](
            pendingCount
        );
        uint256 j = 0;
        for (uint256 i = 0; i < validationSetIDs.length(); i++) {
            ValidationRecord memory record = validationRecords[
                validationSetIDs.at(i)
            ];
            if (record.phase == ValidationPhase.Awaiting) {
                pendingValidations[j] = record;
                j++;
            }
        }
        return pendingValidations;
    }

    function retrieveValidationById(
        bytes32 validationId
    ) public view returns (ValidationRecord memory) {
        if (!validationSetIDs.contains(validationId)) {
            revert ValidationNotFoundError();
        }
        return validationRecords[validationId];
    }

    function initiateValidation(
        AuthorizedMessagePacket[] calldata authorizations
    ) public payable {
        if (authorizations.length == 0) {
            revert EmptyAuthorizationError();
        }

        if (msg.value != validatorParams.DISPUTE_SECURITY_DEPOSIT()) {
            revert InvalidBondAmountError();
        }

        bytes32 validationId = computeValidationId(authorizations);

        if (validationSetIDs.contains(validationId)) {
            revert DuplicateValidationError();
        }

        uint256 targetEpoch = authorizations[0].epoch;
        if (
            targetEpoch >
            _getCurrentEpoch() - validatorParams.FINALIZATION_DELAY_SLOTS()
        ) {
            revert UnfinalizedSegmentError();
        }

        MessageDetails[] memory messagesData = new MessageDetails[](
            authorizations.length
        );
        (
            address msgSender,
            address witnessAuthorizer,
            MessageDetails memory firstMessageData
        ) = recoverAuthorizationData(authorizations[0]);

        messagesData[0] = firstMessageData;

        for (uint256 i = 1; i < authorizations.length; i++) {
            (
                address otherMsgSender,
                address otherAuthorizer,
                MessageDetails memory otherMessageData
            ) = recoverAuthorizationData(authorizations[i]);

            messagesData[i] = otherMessageData;

            if (authorizations[i].epoch != targetEpoch) {
                revert MixedEpochError();
            }
            if (otherMsgSender != msgSender) {
                revert MixedValidatorError();
            }
            if (otherAuthorizer != witnessAuthorizer) {
                revert MixedAuthorizerError();
            }
            if (otherMessageData.sequence != messagesData[i - 1].sequence + 1) {
                revert InvalidSequenceError();
            }
        }

        validationSetIDs.add(validationId);
        validationRecords[validationId] = ValidationRecord({
            attestationId: validationId,
            timestampInit: Time.timestamp(),
            phase: ValidationPhase.Awaiting,
            targetEpoch: targetEpoch,
            validator: msg.sender,
            witnessAuthorizer: witnessAuthorizer,
            protocolDestination: msgSender,
            authorizedMessages: messagesData
        });

        emit ValidationInitiated(validationId, msg.sender, witnessAuthorizer);
    }

    function processTimedOutValidation(bytes32 validationId) public {
        if (!validationSetIDs.contains(validationId)) {
            revert ValidationNotFoundError();
        }

        ValidationRecord storage record = validationRecords[validationId];

        if (record.phase != ValidationPhase.Awaiting) {
            revert ValidationAlreadySettledError();
        }

        if (
            record.timestampInit + validatorParams.CHALLENGE_TIMEOUT_PERIOD() >=
            Time.timestamp()
        ) {
            revert ValidationStillActiveError();
        }

        _finalizeValidation(ValidationPhase.Rejected, record);
    }
}

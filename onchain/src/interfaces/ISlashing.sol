// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

interface ISlashing {
    enum ValidationPhase {
        Awaiting,
        Confirmed,
        Rejected
    }
    struct ValidationRecord {
        bytes32 attestationId;
        uint48 timestampInit;
        ValidationPhase phase;
        uint256 targetEpoch;
        address validator;
        address witnessAuthorizer;
        address protocolDestination;
        MessageDetails[] authorizedMessages;
    }

    struct ParticipantState {
        uint256 sequence;
        uint256 holdings;
    }

    struct AuthorizedMessagePacket {
        bytes payload;
        bytes authorization;
        uint64 epoch;
    }

    struct MessageDetails {
        bytes32 messageDigest;
        uint256 sequence;
        uint256 fuelLimit;
    }

    struct ChainSegmentInfo {
        bytes32 ancestorDigest;
        bytes32 worldStateDigest;
        bytes32 messageTreeDigest;
        uint256 segmentHeight;
        uint256 chronograph;
        uint256 networkFee;
    }

    struct ValidationEvidence {
        uint256 incorporationHeight;
        bytes precedingSegmentRLP;
        bytes incorporationSegmentRLP;
        bytes participantMerkleEvidence;
        bytes[] messageMerkleEvidence;
        uint256[] messagePositions;
    }

    error EmptyAuthorizationError();
    error InvalidBondAmountError();
    error DuplicateValidationError();
    error UnfinalizedSegmentError();
    error MixedEpochError();
    error MixedAuthorizerError();
    error MixedValidatorError();
    error ValidationNotFoundError();
    error ValidationStillActiveError();
    error SegmentTooAgedError();
    error ValidationAlreadySettledError();
    error InvalidSegmentDigestError();
    error InvalidAncestorDigestError();
    error ParticipantNotFoundError();
    error MessageNotFoundError();
    error InvalidMessageEvidenceError();
    error BondTransferFailedError();
    error InvalidEvidenceCountError();
    error ValidationTimedOutError();
    error InvalidSegmentHeightError();
    error InvalidSequenceError();
    error ConsensusRootMissingError();

    event ValidationInitiated(
        bytes32 indexed attestationId,
        address indexed validator,
        address indexed witnessAuthorizer
    );
    event ValidationConfirmed(bytes32 indexed attestationId);
    event ValidationRejected(bytes32 indexed attestationId);
}

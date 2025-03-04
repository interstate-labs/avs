
// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {BLS12381} from "../library/bls/BLS12381.sol";

interface INodeRegistrationSystem {
    struct ValidatorNodeDetails {
        BLS12381.G1Point pubkey;
        string  rpcs;
        bytes20 nodeIdentityHash;
        uint32 gasCapacityLimit;
        address controllerAddress;
    }

    error SignatureVerificationFailed();
    error InvalidOperatorAssignment();
    error SecureRegistrationRequired();

    error InvalidNodeIdentity();


    function fetchValidatorByIdentityHash(
        bytes20 nodeIdentityHash
    ) external view returns (ValidatorNodeDetails memory);

  

}

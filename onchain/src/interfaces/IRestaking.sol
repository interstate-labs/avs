// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {BLS12381} from "../library/bls/BLS12381.sol";

interface IConsensusRestaking {
    error MalformedRequest();
    error ParticipantExists();
    error ParticipantNotFound();
    error NodeProviderNotActive();
    error UnauthorizedProvider();
    error OperationForbidden();
    error AlreadyRegistered();
    error StrategyNotAllowed();
    error NotRegistered();
    error NotOperator();
    error InvalidQuery();

    function getProviderCollateral(
        address provider,
        address tokenAddress
    ) external view returns (uint256);

    function getProviderCollateralTokens(
        address provider
    ) external view returns (address[] memory, uint256[] memory);
}
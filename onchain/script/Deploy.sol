// // SPDX-License-Identifier: MIT
// pragma solidity ^0.8.0;

// import "forge-std/Script.sol";
// import "forge-std/console.sol";
// import "../src/Parameters.sol";
// import "../src/Validator/BaseRegistry.sol";
// import "../src/Registry/ValidatorRegistryBase.sol";
// import "../src/Middleware/Middleware.sol";
// import "../src/Slashing/ValidatorSlashingCore.sol";

// contract DeployScript is Script {
//     function setUp() public {}

//     function run() public {
//         uint256 deployerPrivateKey = vm.envUint("2cb26dcd8b503c3a708448fb27ebd2f725ef1a1305014ec0e44a9f89d204ee0e");
//         vm.startBroadcast(deployerPrivateKey);

//         // Deploy each contract separately and store addresses
//         address parameterAddress = deployParameters();
//         address baseRegistryAddress = deployBaseRegistry();
//         address validatorRegistryAddress = deployValidatorRegistry();
//         address middlewareAddress = deployMiddleware();
//         address slashingCoreAddress = deploySlashingCore();

//         vm.stopBroadcast();

//         // Print deployment summary
//         printDeploymentSummary(
//             parameterAddress,
//             baseRegistryAddress,
//             validatorRegistryAddress,
//             middlewareAddress,
//             slashingCoreAddress
//         );
//     }

//     function deployParameters() internal returns (address) {
//         Parameters parameter = new Parameters();
//         console.log("Parameters deployed at:", address(parameter));
//         return address(parameter);
//     }

//     function deployBaseRegistry() internal returns (address) {
//         BaseRegistry baseRegistry = new BaseRegistry();
//         console.log("BaseRegistry deployed at:", address(baseRegistry));
//         return address(baseRegistry);
//     }

//     function deployValidatorRegistry() internal returns (address) {
//         ValidatorRegistryBase validatorRegistry = new ValidatorRegistryBase();
//         console.log("ValidatorRegistryBase deployed at:", address(validatorRegistry));
//         return address(validatorRegistry);
//     }

//     function deployMiddleware() internal returns (address) {
//         ConsensusEigenLayerMiddleware middleware = new ConsensusEigenLayerMiddleware();
//         console.log("Middleware deployed at:", address(middleware));
//         return address(middleware);
//     }

//     function deploySlashingCore() internal returns (address) {
//         ValidatorSlashingCore slashingCore = new ValidatorSlashingCore();
//         console.log("ValidatorSlashingCore deployed at:", address(slashingCore));
//         return address(slashingCore);
//     }

//     function printDeploymentSummary(
//         address parameterAddress,
//         address baseRegistryAddress,
//         address validatorRegistryAddress,
//         address middlewareAddress,
//         address slashingCoreAddress
//     ) internal view {
//         console.log("\nDeployment Summary:");
//         console.log("-------------------");
//         console.log("Parameters:              ", parameterAddress);
//         console.log("BaseRegistry:            ", baseRegistryAddress);
//         console.log("ValidatorRegistryBase:   ", validatorRegistryAddress);
//         console.log("Middleware:              ", middlewareAddress);
//         console.log("ValidatorSlashingCore:   ", slashingCoreAddress);
//     }
// }
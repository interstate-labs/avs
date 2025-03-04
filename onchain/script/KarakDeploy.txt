// SPDX-License-Identifier: SEE LICENSE IN LICENSE
pragma solidity ^0.8.25;

import {Script, console2} from "forge-std/Script.sol";
import {ERC1967Factory} from "solady/src/utils/ERC1967Factory.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";


import  {TxnVerifier} from "../src/KarakRestaking/KarakRestaking.sol";


import {Core} from "@karak/src/Core.sol";
import {Vault} from "@karak/src/Vault.sol";
import {IDSS} from "@karak/src/interfaces/IDSS.sol";
import {ICore} from "@karak/src/interfaces/ICore.sol";
import {Operator} from "@karak/src/entities/Operator.sol";
import {VaultLib} from "@karak/src/entities/VaultLib.sol";
import {SlashingHandler} from "@karak/src/SlashingHandler.sol";
import {ERC20Mintable} from "@karak/test/helpers/contracts/ERC20Mintable.sol";

contract DeployDSSAndLocal is Script {
    address internal constant CORE_PROXY_ADMIN = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;
    address internal constant CORE_MANAGER = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    address internal constant CORE_VETO_COMMITTEE = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    address internal constant SLASHING_HANDLER_PROXY_ADMIN = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;
    address internal constant OPERATOR = 0x14dC79964da2C08b23698B3D3cc7Ca32193d9955;
    address internal constant AGGREGATOR = 0x49066ddbD166F4A46290076E196AD790A5c0a3F8;

    function run() public {
        vm.startBroadcast();
        console2.log("Running DeployCoreLocal script. Signer:", msg.sender);
        console2.log();

        (address coreImpl, address vaultImpl, address slashingHandlerImpl) = deployImpls();
        console2.log("Deployed Core(impl):", coreImpl);
        console2.log("Deployed Vault(impl):", vaultImpl);
        console2.log("Deployed SlashingHandler(impl):", slashingHandlerImpl);
        console2.log();

        (Core coreProxy, SlashingHandler slashingHandlerProxy) = deployProxies(coreImpl, slashingHandlerImpl);
        console2.log("Deployed Core(proxy):", address(coreProxy));
        console2.log("Deployed SlashingHandler(proxy):", address(slashingHandlerProxy));
        console2.log();

        initializeCore(coreProxy, vaultImpl);
        console2.log("Initialized Core(proxy) with params:");
        console2.log("\tVault Implementation:", vaultImpl);
        console2.log("\tManager:", CORE_MANAGER);
        console2.log("\tVeto Committee:", CORE_VETO_COMMITTEE);
        console2.log();

        ERC20Mintable testERC20 = deployTestERC20();

        testERC20.mint(msg.sender, 1e6 * 1e6);
        console2.log("Deployed TEST ERC20:", address(testERC20));
        console2.log("Minted ", testERC20.balanceOf(msg.sender), " TEST ERC20 to ", msg.sender);
        console2.log();

        IERC20[] memory tokens = new IERC20[](1);
        tokens[0] = IERC20(address(testERC20));

        initializeSlashingHandler(slashingHandlerProxy, tokens);
        console2.log("Initialized SlashingHandler(proxy) with params:");
        console2.log("\tTokens:");
        for (uint256 i = 0; i < tokens.length; i++) {
            console2.log("\t ", i, "-", address(tokens[i]));
        }
        console2.log();

        address operatorERC20Vault =
            setupOperator(address(coreProxy), address(testERC20), address(slashingHandlerProxy), address(vaultImpl));
        console2.log("address of deployed operator contract for ", OPERATOR, " : ", operatorERC20Vault);
        console2.log();

        // SquareNumberDSS dss = deployDSS(address(coreProxy));
        // console2.log("address of DSS: ", address(dss));

        TxnVerifier dssv=deployDSSVerify(address(coreProxy));
        console2.log("address of TxnVerifier",address(dssv));

        operatorRegistrationInCore(address(coreProxy), operatorERC20Vault, dssv, testERC20);
        updateStakeInDSS(coreProxy, operatorERC20Vault, dssv);
        vm.stopBroadcast();

        string memory blockNumberStore = string.concat(
            '{"block_number": 0',
            "}"
        );

        vm.writeFile("./block-number-store.json", blockNumberStore);
    }

    function deployImpls() public returns (address coreImpl, address vaultImpl, address slashingHandlerImpl) {
        coreImpl = address(new Core());
        vaultImpl = address(new Vault());
        slashingHandlerImpl = address(new SlashingHandler());
    }

    function deployProxies(address coreImpl, address slashingHandlerImpl)
        public
        returns (Core coreProxy, SlashingHandler slashingHandlerProxy)
    {
        ERC1967Factory factory = new ERC1967Factory();
        coreProxy = Core(factory.deploy(coreImpl, CORE_PROXY_ADMIN));
        slashingHandlerProxy = SlashingHandler(factory.deploy(slashingHandlerImpl, SLASHING_HANDLER_PROXY_ADMIN));
    }

    function initializeCore(Core core, address vaultImpl) public {
        core.initialize(vaultImpl, CORE_MANAGER, CORE_VETO_COMMITTEE, 100000, 100000, 10000);
    }

    function deployTestERC20() public returns (ERC20Mintable testERC20) {
        testERC20 = new ERC20Mintable();
        testERC20.initialize("Test", "TEST", 6);
    }

    function initializeSlashingHandler(SlashingHandler slashingHandler, IERC20[] memory tokens) public {
        slashingHandler.initialize(msg.sender, tokens);
    }

    function setupOperator(address coreProxy, address erc20Token, address slashingHandler, address vaultImpl)
        public
        returns (address vaultAddress)
    {
        address[] memory assets = new address[](1);
        assets[0] = erc20Token;
        address[] memory slashingHandlers = new address[](1);
        slashingHandlers[0] = slashingHandler;

        Core(coreProxy).allowlistAssets(assets, slashingHandlers);

        VaultLib.Config[] memory configs = new VaultLib.Config[](1);
        configs[0] = VaultLib.Config({
            asset: erc20Token,
            decimals: 6,
            operator: OPERATOR,
            name: "Test",
            symbol: "TEST",
            extraData: abi.encode("")
        });
        vaultAddress = address(Core(coreProxy).deployVaults(configs, vaultImpl)[0]);
    }

    // function deployDSS(address core) public returns (SquareNumberDSS dss) {
    //     dss = new SquareNumberDSS(AGGREGATOR, ICore(core));
    //     dss.registerToCore(1);
    // }

        function deployDSSVerify(address core) public returns (TxnVerifier dssv) {
        dssv = new TxnVerifier(AGGREGATOR, ICore(core));
        dssv.registerToCore(1);
    }


    function operatorRegistrationInCore(
        address core,
        address vaultAddress,
        TxnVerifier dss,
        ERC20Mintable testERC20
    ) public {
        testERC20.approve(vaultAddress, 10000);
        Vault(vaultAddress).deposit(1000, msg.sender);
        Core(core).registerOperatorToDSS(IDSS(address(dss)), abi.encode(""));
    }

    function updateStakeInDSS(Core core, address vaultAddress, TxnVerifier dss) public {
        core.requestUpdateVaultStakeInDSS(
            Operator.StakeUpdateRequest({vault: vaultAddress, dss: IDSS(address(dss)), toStake: true})
        );
        // core.finalizeUpdateVaultStakeInDSS(updatedStake, OPERATOR);
    }
}

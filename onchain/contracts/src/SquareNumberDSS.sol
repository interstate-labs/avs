// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {IDSS} from "./karak/src/interfaces/IDSS.sol";
import {ICore} from "./karak/src/interfaces/ICore.sol";
import {Operator} from "./karak/src/entities/Operator.sol";

contract SquareNumberDSS is IDSS {
    struct TaskRequest {
        uint256 value;
    }

    struct TaskResponse {
        uint256 response;
    }

    constructor(address _aggregator, ICore _core) {
        aggregator = _aggregator;
        core = _core;
    }

    /* ======= State Variables ======= */

    mapping(address operatorAddress => bool exists) operatorExists;
    address[] operatorAddresses;
    address aggregator;
    mapping(bytes32 taskRequestHash => bool exists) taskExists;
    mapping(bytes32 taskRequestHash => bool completed) taskCompleted;
    mapping(bytes32 taskRequestHash => TaskResponse taskResponse) taskResponses;
    ICore core;

    /* ======= Events ======= */

    event TaskRequestGenerated(address sender, TaskRequest taskRequest);
    event TaskResponseSubmitted(TaskResponse taskResponse);

    /* ======= External Functions ======= */

    function slashOperator(address operator, uint256 index) external {}

    function generateTaskRequest(TaskRequest calldata taskRequest) external {
        bytes32 taskRequestHash = keccak256(abi.encode(taskRequest));
        if (taskExists[taskRequestHash]) revert TaskAlreadyExists();
        taskExists[taskRequestHash] = true;
        emit TaskRequestGenerated(msg.sender, taskRequest);
    }

    function registerToCore(uint256 slashablePercentage) external {
        core.registerDSS(slashablePercentage);
    }

    /* ======= Hooks ======= */

    function supportsInterface(bytes4 interfaceID) external pure returns (bool) {
        if (interfaceID == IDSS.registrationHook.selector || interfaceID == IDSS.unregistrationHook.selector) {
            return true;
        }
        return false;
    }

    function registrationHook(address operator, bytes memory extraData) external senderIsOperator(operator) {
        extraData = extraData;
        if (operatorExists[operator]) revert OperatorAlreadyRegistered();
        operatorAddresses.push(operator);
        operatorExists[operator] = true;
    }

    function unregistrationHook(address operator, bytes memory extraData) external senderIsOperator(operator) {
        uint256 index = abi.decode(extraData, (uint256));
        if (operator != operatorAddresses[index]) revert OperatorAndIndexDontMatch();
        if (!operatorExists[operator]) revert OperatorIsNotRegistered();
        uint256 operatorAddressesLength = operatorAddresses.length;
        operatorAddresses[index] = operatorAddresses[operatorAddressesLength - 1];
        operatorAddresses.pop();
        operatorExists[operator] = false;
    }

    function requestUpdateStakeHook(address operator, Operator.StakeUpdateRequest memory newStake) external {}
    function cancelUpdateStakeHook(address operator, address vault) external {}
    function finishUpdateStakeHook(address operator) external {}
    function requestSlashingHook(address operator, uint256[] memory slashingPercentagesWad) external {}
    function cancelSlashingHook(address operator) external {}
    function finishSlashingHook(address operator) external {}

    /* ======= Only Aggregator Functions ======= */

    function submitTaskResponse(TaskRequest calldata taskRequest, TaskResponse calldata taskResponse)
        external
        onlyAggregator
    {
        bytes32 taskReqeustHash = keccak256(abi.encode(taskRequest));
        taskCompleted[taskReqeustHash] = true;
        taskResponses[taskReqeustHash] = taskResponse;
        emit TaskResponseSubmitted(taskResponse);
    }

    /* ======= View Functions ======= */

    function getTaskResponse(TaskRequest calldata taskRequest) external view returns (TaskResponse memory) {
        bytes32 taskRequestHash = keccak256(abi.encode(taskRequest));
        return taskResponses[taskRequestHash];
    }

    function isTaskCompleted(TaskRequest calldata taskRequest) external view returns (bool) {
        bytes32 taskRequestHash = keccak256(abi.encode(taskRequest));
        return taskCompleted[taskRequestHash];
    }

    function isOperatorRegistered(address operator) external view returns (bool) {
        return operatorExists[operator];
    }

    /* ======= Modifiers ======= */
    modifier onlyAggregator() {
        if (msg.sender != aggregator) revert NotAggregator();
        _;
    }

    modifier senderIsOperator(address operator) {
        if (tx.origin != operator) revert SenderNotOperator();
        _;
    }

    /* ======= Errors ======= */
    error OperatorAlreadyRegistered();
    error OperatorAndIndexDontMatch();
    error OperatorIsNotRegistered();
    error NotAggregator();
    error TaskAlreadyExists();
    error SenderNotOperator();
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IDSS} from "@karak/src/interfaces/IDSS.sol";
import {ICore} from "@karak/src/interfaces/ICore.sol";
import {Operator} from "@karak/src/entities/Operator.sol";



contract TxnVerifier is IDSS {
    // Event to emit the verification result
    // event TxnVerificationResult(bytes32 txnHash, uint256 blockNumber);
      event TxnVerificationResult(string pubkey, bytes32 txnHash, uint256 blockNumber);
    
    
    event TaskResponseSubmitted(OperatorResponse taskResponse);
    
    // Store verified transactions
    // mapping(bytes32 => uint256) public verifiedTxns;
      mapping(string => mapping(bytes32 => uint256)) public verifiedTxns;
      mapping(address operatorAddress => bool exists) operatorExists;
    
    mapping(bytes32 => bool) public taskCompleted;
    // Aggregator address
    address public aggregator;
    address public owner;
    ICore core;
    

    struct OperatorResponse {
        bool is_included;
        uint64 proposer_index;
        string block_number;    
    }
    

    struct Task {
        string pubkey;
        string transaction_hash;
        string block_number;
    }
    
    // State variables to store txnValid and proposer
    bool private txnValid;
    uint256 private proposer;
    
    // Added missing mapping
    mapping(bytes32 => OperatorResponse) public taskResponses;
     address[] operatorAddresses;
    
    /* ======= Modifiers ======= */
    modifier onlyAggregator() {
        require(msg.sender == aggregator, "Not Aggregator");
        _;
    }

     modifier senderIsOperator(address operator) {
        if (tx.origin != operator) revert SenderNotOperator();
        _;
    }

       modifier onlyCore() {
        require(msg.sender == address(core), "Not Core");
        _;
    }

    modifier onlyOwner(){
            require(msg.sender == address(owner), "Not Owner");
        _;

    }
    
    constructor(address _aggregator, ICore _core,address _owner) {
        aggregator = _aggregator;
        core = _core;
        owner=_owner
    }
    
    /* ======= External Functions ======= */
    
    // Function to verify transaction inclusion
    function verifyTransaction(
        string calldata pubkey, 
        bytes32 txnHash,
        uint256 blockNumber
    ) public {
        // Simulate verification logic here (placeholder)
        // verifiedTxns[txnHash] = blockNumber;

        verifiedTxns[pubkey][txnHash] = blockNumber;
        

        
        // Emit the verification result
        // emit TxnVerificationResult(txnHash, blockNumber);
        emit TxnVerificationResult(pubkey, txnHash, blockNumber);
    }
    
    // function submitTaskResponse(string calldata pubkey ,string calldata taskRequest, OperatorResponse calldata taskResponse)   
    //     external
    //     onlyAggregator
    // {
    //     bytes32 taskRequestHash = keccak256(abi.encode(taskRequest));  
    //     taskCompleted[taskRequestHash] = true;
    //     taskResponses[taskRequestHash] = taskResponse;
    //     emit TaskResponseSubmitted(taskResponse);
    // }
 
 function submitTaskResponse(
    string calldata pubkey,
    string calldata taskRequest,
    OperatorResponse calldata taskResponse
) external onlyAggregator {
    // Hash pubkey and taskRequest together to create a unique identifier
    bytes32 taskRequestHash = keccak256(abi.encode(pubkey, taskRequest));  

    // Store the task completion status and response
    taskCompleted[taskRequestHash] = true;
    taskResponses[taskRequestHash] = taskResponse;

    // Emit event with the response
    emit TaskResponseSubmitted(taskResponse);
}



    function getTaskResponseVerifiy(Task calldata taskRequest) external view returns (OperatorResponse memory) {
        bytes32 taskRequestHash = keccak256(abi.encode(taskRequest));
        return taskResponses[taskRequestHash];
    }

    function getTaskResponse(string calldata pubkey ,string calldata taskRequest) external view returns (OperatorResponse memory) {
        // bytes32 taskRequestHash = keccak256(abi.encode(taskRequest));
          bytes32 taskRequestHash = keccak256(abi.encode(pubkey, taskRequest));  

        return taskResponses[taskRequestHash];
    }

     function isOperatorRegistered(address operator) external view returns (bool) {
        return operatorExists[operator];
    }

    
    /* ======= IDSS Interface Functions ======= */
    
    function supportsInterface(
        bytes4 interfaceID
    ) external pure returns (bool) {
        return (interfaceID == IDSS.registrationHook.selector ||
            interfaceID == IDSS.unregistrationHook.selector);
    }
    
    function registerToCore(uint256 slashablePercentage) onlyOwner external {
        core.registerDSS(slashablePercentage);
    }
    
  function registrationHook(address operator, bytes memory extraData) external  onlyCore senderIsOperator(operator) {
        extraData = extraData;
        if (operatorExists[operator]) revert OperatorAlreadyRegistered();
        operatorAddresses.push(operator);
        operatorExists[operator] = true;
    }


    
    function unregistrationHook(address operator) external onlyCore senderIsOperator(operator) {
    uint256 operatorAddressesLength = operatorAddresses.length;
    for (uint256 i = 0; i < operatorAddressesLength; i++) {
        if (operatorAddresses[i] == operator) {
            // Swap and pop pattern to remove the operator
            operatorAddresses[i] = operatorAddresses[operatorAddressesLength - 1];
            operatorAddresses.pop();
            break;
        }
    }
    
    // Update the mapping regardless of whether operator was found
    operatorExists[operator] = false;
    }

    
    function requestUpdateStakeHook(
        address operator,
        Operator.StakeUpdateRequest memory newStake
    ) external override {}
    
    function cancelUpdateStakeHook(
        address operator,
        address vault
    ) external override {}
    
    function finishUpdateStakeHook(address operator) external override {}
    
    function requestSlashingHook(
        address operator,
        uint256[] memory slashingPercentagesWad
    ) external override {}
    
    function cancelSlashingHook(address operator) external override {}
    
    function finishSlashingHook(address operator) external override {}

     error SenderNotOperator();
      error OperatorAlreadyRegistered();
      error OperatorAndIndexDontMatch();
      error OperatorIsNotRegistered();
}
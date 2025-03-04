# Square Number DSS

The Square Number DSS serves as a fundamental example, specifically crafted to illustrate the basic interactions between the Distributed Secure Service (DSS) and the Karak Restaking mechanism. 

It provides a straightforward, clear-cut case that showcases how users, aggregators, and operators within the DSS ecosystem work together to process a computational task. This example is essential for understanding the core principles of how DSS utilizes economic security from restaked tokens, handles task requests, and ensures accurate and reliable results through collective validation and the potential for operator accountability.

## Installation and Setup

### Prerequisites
- docker engine installed and running on your machine - https://docs.docker.com/engine/install/
- docker compose installed - https://docs.docker.com/compose/install/
- Availability of ports 8080, 8081, 8454, 3000 (You can change the ports in docker-compose.yaml if needed)

### Running the Containers

`docker-compose up --build`

#### Containers:
- anvil
- contract-deployer
- operator-1
- operator-2
- aggregator

### Calling the DSS
When the DSS is deployed you can check the address in the `contracts/contract-addresses.json` file. 

Using those addresses you can make the following cast call to submit a task request for squaring a number:
```bash 
cast send 0x9A9f2CCfdE556A7E9Ff0848998Aa4a0CFD8863AE "generateTaskRequest((uint256))" "(5)" --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d --rpc-url http://localhost:8545
```

Wait for about 10-20 seconds to allow the operators to pick up and finish the task and then using a cast call you can check the result as follows:
```bash
cast call 0x9A9f2CCfdE556A7E9Ff0848998Aa4a0CFD8863AE "getTaskResponse((uint256))(uint256)" "(5)" --rpc-url http://localhost:8545
```

## Architecture
![Square Number DSS](illustrations/architecture.svg)

- **Aggregator**: Acts as a trusted central entity within the DSS, monitoring task requests emitted by the DSS contract. It distributes these requests to all registered operators.

- **Operators**: Perform computational tasks upon receiving requests from the aggregator. They calculate the square of the number, sign the result, and send it back to the aggregator for validation.

#### User Task Request
A user initiates the process by generating a task request to square a number. This request is made through a contract call to the DSS contract, effectively registering the task within the system.

The aggregator, functioning as an offline entity within the DSS, acts as a trusted central figure. Its primary role is to monitor the DSS contract for any new task requests. As soon as a task request is detected, the aggregator disseminates this request to all operators registered in the DSS.

Upon receiving the task request, each operator performs the computation to square the given number. After calculating the square, the operator signs the result and sends it back to the aggregator. This ensures that each response is authenticated and traceable to its origin.

The aggregator collects all responses from the operators and verifies their signatures to confirm that the responses are genuinely from the registered operators. Once verified, the aggregator calculates the median of all the received responses. The median is chosen to mitigate the impact of any outliers or erroneous calculations.

Finally, the aggregator posts the __stake based median__ value of the squared number to the DSS contract. This final step completes the task request cycle, ensuring that the user receives a reliable and accurate result, backed by the collective validation of multiple operators.

#### Slashing Mechanism
Although the slashing logic is not yet implemented, it is designed to allow the DSS to penalize any operator who fails to meet performance expectations. If an operator does not perform as required, the DSS has the authority to slash a portion of that operator's staked tokens. This potential for slashing serves as a powerful incentive for operators to maintain integrity and deliver consistent, high-quality performance.

#### Ensuring Integrity and Reliability
By utilizing this architecture, the DSS maintains a robust and secure method for processing computational tasks, reinforced by the economic security provided through Karak Restaking. The threat of slashing staked tokens for poor performance incentivizes operators to maintain high standards of integrity and reliability.

## Flow Diagram
The following flow diagram should be able to give you a brief overview of the entire working of the square number DSS.
![Square Number DSS Flow](illustrations/flow.svg)

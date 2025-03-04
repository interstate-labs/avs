use alloy::sol;

sol!(
    #[sol(rpc)]
    SquareNumberDSS,
    "/app/abi/SquareNumberDSS.json",
);

sol!(
    #[sol(rpc)]
    TxnVerifier,
    "/app/abi/TxnVerifier.json",
);


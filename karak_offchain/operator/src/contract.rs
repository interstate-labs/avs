use alloy::sol;

sol!(
    #[sol(rpc)]
    SquareNumberDSS,
    "../abi/SquareNumberDSS.json",
);

sol!(
    #[sol(rpc)]
    TxnVerifier,
    "../abi/TxnVerifier.json",
);


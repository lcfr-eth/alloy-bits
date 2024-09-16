/*
extra gas is refunded to the sender. This can be used to mine a tx hash that starts with a specific prefix
by increasing the gas limit from X until the tx hash starts with the desired prefix.
*/

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{U256, Bytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    consensus::TxEnvelope,
    sol,
    sol_types::SolCall,
};
use eyre::Result;
use hex;
use tokio;

sol!(
    #[allow(missing_docs)]
    function setName(string calldata s) public;
);

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "http://127.0.0.1:8545".parse()?;

    let key = std::env::var("PK")
        .expect("PK environment variable not set");

    let signer: PrivateKeySigner = key.parse()?;
    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet.clone())
        .on_http(rpc_url);

    let nonce = provider.get_transaction_count(signer.address()).await?;
    let eip1559_est = provider.estimate_eip1559_fees(None).await?;

    let call = setNameCall { s: "poop".to_string() }.abi_encode();
    let input = Bytes::from(call); 

    let mut tx_envelope;
    let mut gas: u64 = 100_000;

    loop {
        let tx = TransactionRequest::default()
            .with_to("0x5FbDB2315678afecb367f032d93F642f64180aa3".parse()?)
            .with_nonce(nonce)
            .with_chain_id(31337)
            .with_value(U256::from(0))
            .with_gas_limit(gas.into())
            .with_max_priority_fee_per_gas(eip1559_est.max_priority_fee_per_gas)
            .with_max_fee_per_gas(eip1559_est.max_fee_per_gas)
            .with_input(input.clone());

        tx_envelope = tx.build(&wallet).await?;

        if let TxEnvelope::Eip1559(ref signed_tx) = tx_envelope {
            let tx_hash = hex::encode(signed_tx.hash());

            if tx_hash.starts_with("dead") {
                println!("Found a transaction hash starting with prefix: {:?}", tx_hash);
                println!("Gas used: {}", gas);
                //let receipt = provider.send_tx_envelope(tx_envelope).await?.get_receipt().await?;
                //println!("Sent transaction: {}", receipt.transaction_hash);
                break;
            }
        }
        gas += 1;
    }

    Ok(())
}

// mine tx by increasing (starting at baseFee) the max fee per gas until the tx hash starts with "dead"

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    consensus::TxEnvelope,
};
use eyre::Result;
use hex;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "http://10.0.0.226:8545".parse()?;

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

    let mut max_fee_per_gas = eip1559_est.max_fee_per_gas;
    println!("Starting BaseFee: {}", max_fee_per_gas);

    let mut tx_envelope;
    loop {
        let tx = TransactionRequest::default()
            .with_to("0x328eBc7bb2ca4Bf4216863042a960E3C64Ed4c10".parse()?)
            .with_nonce(nonce)
            .with_chain_id(1)
            .with_value(U256::from(0))
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(eip1559_est.max_priority_fee_per_gas)
            .with_max_fee_per_gas(max_fee_per_gas.into());

        tx_envelope = tx.build(&wallet).await?;

        if let TxEnvelope::Eip1559(ref signed_tx) = tx_envelope {
            let tx_hash = hex::encode(signed_tx.hash());

            if tx_hash.starts_with("dead") {
                println!("Found a transaction hash starting with prefix: {:?}", tx_hash);
                println!("max fee per gas: {}", max_fee_per_gas);

                //let receipt = provider.send_tx_envelope(tx_envelope).await?.get_receipt().await?;
                //println!("Sent transaction: {}", receipt.transaction_hash);
                break;
            }
        } 

        max_fee_per_gas += 1;
    }

    Ok(())
}

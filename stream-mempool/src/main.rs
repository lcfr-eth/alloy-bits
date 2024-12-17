use eyre::Result;
use futures_util::StreamExt;

use alloy::{
    network::TransactionBuilder,
    primitives::{address, U256},
    providers::{ext::TraceApi, ProviderBuilder},
    rpc::types::{trace::parity::TraceType, TransactionRequest, Transaction},
};
use alloy::providers::Provider;
use alloy::providers::WsConnect;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "ws://127.0.0.1:8546";

    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let sub = provider.subscribe_full_pending_transactions().await?;

    // Wait and take the next 3 transactions.
    //let mut stream = sub.into_stream();
    let mut stream = sub.into_stream().take(1);

    println!("Awaiting pending transactions...");

    let handle = tokio::spawn(async move {
        while let Some(tx) = stream.next().await {
            //println!("{:#?}", tx);

	    //let tx = TransactionRequest::default().from(tx.from).to(tx.transact_to).input(calldata.into());

            //TransactionRequest::default().with_from(tx.from).with_to(tx.transact_to).with_data(tx.data.0);


	    let trace_type = [TraceType::Trace];
            let result = provider.trace_call(&tx.into(), &trace_type).await;
            println!("{:?}", result.unwrap().trace);
      }
    });

    handle.await?;

    Ok(())
}


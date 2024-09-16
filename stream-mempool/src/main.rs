
use alloy_provider::{network::AnyNetwork, Provider, ProviderBuilder, RootProvider};
use alloy_rpc_client::WsConnect;
use eyre::Result;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "ws://10.0.0.227:8546";

    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let sub = provider.subscribe_full_pending_transactions().await?;

    // Wait and take the next 3 transactions.
    let mut stream = sub.into_stream();

    println!("Awaiting pending transactions...");

    let handle = tokio::spawn(async move {
        while let Some(tx) = stream.next().await {
            println!("{:#?}", tx);
        }
    });

    handle.await?;

    Ok(())
}


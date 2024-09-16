//! Example of subscribing and listening for pending transactions in the public mempool by
//! `WebSocket` subscription.

// use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy_provider::{network::AnyNetwork, Provider, ProviderBuilder, RootProvider};
use alloy_rpc_client::WsConnect;
use eyre::Result;
use futures_util::StreamExt;
use alloy_rpc_types::{Block, BlockId, BlockNumberOrTag};

use alloy_rpc_types_trace::{
    filter::TraceFilter,
    parity::{LocalizedTransactionTrace, TraceResults, TraceResultsWithTransactionHash, TraceType},
};

use alloy_provider::ext::TraceApi;
use alloy_rpc_types_trace::parity::Action;

#[tokio::main]
async fn main() -> Result<()> {
    let ws_url = "ws://10.0.0.227:8546";

    let ws = WsConnect::new(ws_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;


    let subscription = provider.subscribe_blocks().await?;
    let mut stream = subscription.into_stream();

    while let Some(block) = stream.next().await {
        println!(
            "Received block number: {}",
            block.header.number.expect("Failed to get block number")
        );

        let traces = provider.trace_block(BlockNumberOrTag::Number(block.header.number.expect("REASON"))).await?;
        // TransportResult<Vec<LocalizedTransactionTrace>>

        for trace in traces {
            //println!("Trace: {:?}", trace);
            match trace.trace.action {
                Action::Call(tx) => {
                    // if tx.input < 4 then return
                    if tx.input.0.len() < 4 {
                        continue;
                    }
                },
                _ => {}
            }
        }

    }

    Ok(())
}
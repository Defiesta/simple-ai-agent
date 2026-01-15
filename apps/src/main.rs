// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::Duration;

use crate::trading_signal::ITradingSignal::ITradingSignalInstance;
use alloy::{
    primitives::{Address, U256},
    signers::local::PrivateKeySigner,
    sol_types::SolValue,
};
use anyhow::{bail, Context, Result};
use boundless_market::{Client, Deployment, StorageProviderConfig};
use clap::Parser;
use guests::TRADING_SIGNAL_ELF;
use url::Url;

/// Timeout for the transaction to be confirmed.
pub const TX_TIMEOUT: Duration = Duration::from_secs(30);

mod trading_signal {
    alloy::sol!(
        #![sol(rpc, all_derives)]
        "../contracts/src/ITradingSignal.sol"
    );
}

/// Arguments of the trading signal CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Current ETH price in wei for prediction input.
    #[clap(long, default_value = "3700000000000000000")]
    current_price: u64,
    /// URL of the Ethereum RPC endpoint.
    #[clap(short, long, env)]
    rpc_url: Url,
    /// Private key used to interact with contracts and the Boundless Market.
    #[clap(long, env)]
    private_key: PrivateKeySigner,
    /// Address of the TradingSignal contract.
    #[clap(long, env)]
    trading_signal_address: Address,
    /// URL where provers can download the program to be proven.
    #[clap(long, env)]
    program_url: Option<Url>,
    /// Submit the request offchain via the provided order stream service url.
    #[clap(short, long, requires = "order_stream_url")]
    offchain: bool,
    /// Configuration for the StorageProvider to use for uploading programs and inputs.
    #[clap(flatten, next_help_heading = "Storage Provider")]
    storage_config: StorageProviderConfig,
    /// Deployment of the Boundless contracts and services to use.
    ///
    /// Will be automatically resolved from the connected chain ID if unspecified.
    #[clap(flatten, next_help_heading = "Boundless Market Deployment")]
    deployment: Option<Deployment>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match dotenvy::dotenv() {
        Ok(path) => tracing::debug!("Loaded environment variables from {:?}", path),
        Err(e) if e.not_found() => tracing::debug!("No .env file found"),
        Err(e) => bail!("failed to load .env file: {}", e),
    }
    let args = Args::parse();

    // Create a Boundless client from the provided parameters.
    let client = Client::builder()
        .with_rpc_url(args.rpc_url.clone())
        .with_deployment(args.deployment.clone())
        .with_storage_provider_config(&args.storage_config)?
        .with_private_key(args.private_key.clone())
        .build()
        .await
        .context("failed to build boundless client")?;

    run_trading_signal_mode(&args, &client).await?;

    Ok(())
}

async fn run_trading_signal_mode(args: &Args, client: &Client) -> Result<()> {
    let contract_address = args.trading_signal_address;
    let current_price = args.current_price;

    tracing::info!("Current ETH price: {} wei ({:.2} ETH)", current_price, current_price as f64 / 1e18);
    let input_bytes = U256::from(current_price).abi_encode();

    // Build the request based on whether program URL is provided
    let request = if let Some(program_url) = &args.program_url {
        // Use the provided URL
        client
            .new_request()
            .with_program_url(program_url.clone())?
            .with_stdin(input_bytes.clone())
    } else {
        client
            .new_request()
            .with_program(TRADING_SIGNAL_ELF)
            .with_stdin(input_bytes)
    };

    let (request_id, expires_at) = client.submit_onchain(request).await?;

    // Wait for the request to be fulfilled
    tracing::info!("Waiting for trading signal request {:x} to be fulfilled", request_id);
    let fulfillment = client
        .wait_for_request_fulfillment(
            request_id,
            Duration::from_secs(5), // check every 5 seconds
            expires_at,
        )
        .await?;
    tracing::info!("Request {:x} fulfilled", request_id);

    // Decode the journal data from the guest program
    let data = &fulfillment.fulfillmentData;
    tracing::info!("Raw fulfillment data length: {} bytes", data.len());
    tracing::info!("Fulfillment data hex: {}", hex::encode(data));
    
    // The guest now uses proper ABI encoding for (uint8, uint256, uint256)
    // Boundless wraps our journal data in a fulfillment structure
    // Based on the hex data, our 96-byte journal starts at offset 128 (0x80)
    let output: (U256, U256, U256) = if data.len() >= 224 {
        // Extract our journal from the Boundless fulfillment data
        // The structure is: [32-byte offset][32-byte IMAGE_ID][32-byte offset][32-byte offset][96-byte journal]
        let journal_start = 128; // Offset where our journal data starts
        let journal_data = &data[journal_start..journal_start + 96];
        
        tracing::info!("Extracting journal from offset {} (96 bytes)", journal_start);
        tracing::debug!("Journal hex: {}", hex::encode(journal_data));
        
        if let Ok(decoded_tuple) = <(U256, U256, U256)>::abi_decode(journal_data) {
            tracing::info!("Successfully decoded journal data as ABI tuple");
            decoded_tuple
        } else {
            tracing::error!("Failed to decode journal data at expected offset");
            // Return safe fallback values
            (U256::from(0u8), U256::from(0u64), U256::from(3700000000000000000u64))
        }
    } else if data.len() >= 96 {
        // Fallback: try to find a 96-byte window that decodes properly
        let mut found_tuple = None;
        
        for start_offset in (0..=data.len().saturating_sub(96)).step_by(32) {
            let potential_tuple_data = &data[start_offset..start_offset + 96];
            if let Ok(decoded_tuple) = <(U256, U256, U256)>::abi_decode(potential_tuple_data) {
                // Validate that this looks like reasonable trading data
                let signal = decoded_tuple.0.as_limbs()[0] as u8;
                let confidence = decoded_tuple.1.as_limbs()[0];
                let price = decoded_tuple.2.as_limbs()[0];
                
                if signal <= 1 && confidence <= 100 && price > 100_000_000_000_000_000 { // > 0.1 ETH
                    found_tuple = Some(decoded_tuple);
                    tracing::info!("Found valid ABI tuple at offset {}: signal={}, confidence={}, price={}", 
                                  start_offset, signal, confidence, price);
                    break;
                }
            }
        }
        
        found_tuple.unwrap_or_else(|| {
            tracing::error!("Failed to decode fulfillment data as valid trading signal tuple");
            (U256::from(0u8), U256::from(0u64), U256::from(3700000000000000000u64))
        })
    } else {
        tracing::error!("Fulfillment data too short, expected at least 96 bytes but got {}", data.len());
        (U256::from(0u8), U256::from(0u64), U256::from(3700000000000000000u64))
    };
    
    // Debug: Print raw decoded values
    tracing::info!("Raw decoded values: signal={}, confidence={}, predicted_price={}", 
                   output.0, output.1, output.2);
    
    let signal = output.0.as_limbs()[0] as u8;
    let confidence = output.1.as_limbs()[0];
    let predicted_price = output.2.as_limbs()[0];
    
    // Debug: Print converted values  
    tracing::info!("Converted values: signal={}, confidence={}, predicted_price={}", 
                   signal, confidence, predicted_price);

    let action_str = if signal == 1 { "BUY" } else { "SELL" };
    tracing::info!(
        "Trading Signal: {} ETH (confidence: {}%, predicted price: {} wei / {:.2} ETH)",
        action_str,
        confidence,
        predicted_price,
        predicted_price as f64 / 1e18
    );

    // Interact with the TradingSignal contract
    let trading_signal = ITradingSignalInstance::new(contract_address, client.provider().clone());
    let call_set = trading_signal
        .setSignal(signal, U256::from(confidence), U256::from(predicted_price), fulfillment.seal)
        .from(client.caller());

    tracing::info!("Calling TradingSignal setSignal function");
    let pending_tx = call_set.send().await.context("failed to broadcast tx")?;
    tracing::info!("Broadcasting tx {}", pending_tx.tx_hash());
    let tx_hash = pending_tx
        .with_timeout(Some(TX_TIMEOUT))
        .watch()
        .await
        .context("failed to confirm tx")?;
    tracing::info!("Tx {:?} confirmed", tx_hash);

    // Query the stored signal
    let latest_signal = trading_signal
        .getLatestSignal()
        .call()
        .await
        .context("failed to get latest signal from contract")?;
    
    let action_display = if latest_signal.action == 1 { "BUY" } else { "SELL" };
    tracing::info!(
        "Contract updated - Action: {}, Confidence: {}%, Predicted: {} wei ({:.2} ETH), Timestamp: {}",
        action_display,
        latest_signal.confidence,
        latest_signal.predictedPrice.as_limbs()[0],
        latest_signal.predictedPrice.as_limbs()[0] as f64 / 1e18,
        latest_signal.timestamp
    );

    Ok(())
}

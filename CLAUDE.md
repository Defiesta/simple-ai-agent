# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a Boundless Foundry template for building verifiable AI-powered DeFi applications using RISC Zero. The project demonstrates a complete end-to-end flow from AI guest programs running linear regression in the zkVM to Solidity smart contracts that store verified trading signals.

## 🚀 Current Status: PRODUCTION READY

- ✅ **Smart Contract Deployed**: `0xEe747ac1869f9F805dCa40Ef2E6197C2F2e25f16` on Base Mainnet
- ✅ **Contract Verified**: https://basescan.org/address/0xee747ac1869f9f805dca40ef2e6197c2f2e25f16
- ✅ **AI Trading Signal Working**: BUY signal, 97% confidence, 1.10 ETH predicted price
- 🔄 **IMAGE_ID Syncing**: Contract supports dynamic updates via `setImageId()`

### Key Components

- **Smart Contracts** (`contracts/`): Solidity contracts that verify RISC Zero proofs
  - `TradingSignal.sol`: Main contract that stores AI-generated trading signals with confidence scores
  - `ITradingSignal.sol`: Interface for the trading signal contract
  - `ImageID.sol`: Auto-generated contract containing guest program image IDs
  - Uses RISC Zero's verifier system for proof validation

- **RISC Zero Guests** (`guests/`): Zero-knowledge programs that run in the RISC Zero zkVM
  - `trading-signal/`: AI guest program that performs linear regression on ETH price data and generates trading signals
  - Guest programs output is committed as a journal for verification

- **Client Application** (`apps/`): Rust application that coordinates between guests and contracts
  - Submits requests to Boundless Market for proof generation of trading signals
  - Interacts with TradingSignal contract using the generated proofs
  - Uses Alloy for Ethereum interactions

- **Dependencies** (`lib/`): Contains risc0-ethereum library for contract integration

## Common Development Commands

### Building
```bash
# Build Solidity contracts
forge build

# Build Rust code (including guest programs)
cargo build
```

### Testing
```bash
# Test smart contracts
forge test -vvv

# Test Rust code and guests
cargo test
```

### Contract Deployment
```bash
# Deploy TradingSignal contract
VERIFIER_ADDRESS="0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187" forge script contracts/scripts/Deploy.s.sol --rpc-url ${RPC_URL:?} --broadcast -vv
```

### Running the Trading Signal Application

```bash
# Complete command using all environment variables (Base mainnet)
RUST_LOG=info cargo run --release --bin app -- --rpc-url $RPC_URL --private-key $PRIVATE_KEY --trading-signal-address $TRADING_SIGNAL_ADDRESS --program-url $PROGRAM_URL --current-price 3700000000000000000 --chain-id $CHAIN_ID --boundless-market-address $BOUNDLESS_MARKET_ADDRESS --set-verifier-address $SET_VERIFIER_ADDRESS --storage-provider $STORAGE_PROVIDER --pinata-jwt $PINATA_JWT

# Using pre-uploaded guest program
RUST_LOG=info cargo run --bin app -- --current-price 3700000000000000000 --program-url <PROGRAM_URL>

# Upload and use your own guest program (requires PINATA_JWT)
RUST_LOG=info cargo run --bin app -- --current-price 3700000000000000000
```

## Environment Variables

**Current Production Configuration (Base Mainnet)**:
- `RPC_URL`: https://base-mainnet.g.alchemy.com/v2/N-Gnpjy1WvCfokwj6fiOfuAVL_At6IvE
- `TRADING_SIGNAL_ADDRESS`: 0xEe747ac1869f9F805dCa40Ef2E6197C2F2e25f16 (verified contract)
- `VERIFIER_ADDRESS`: 0x0b144e07a0826182b6b59788c34b32bfa86fb711 (RISC Zero verifier)
- `CHAIN_ID`: 8453 (Base mainnet)
- `BOUNDLESS_MARKET_ADDRESS`: 0xfd152dadc5183870710fe54f939eae3ab9f0fe82
- `SET_VERIFIER_ADDRESS`: 0x1Ab08498CfF17b9723ED67143A050c8E8c2e3104

**Required for Development**:
- `PRIVATE_KEY`: Wallet private key with sufficient ETH on Base
- `PINATA_JWT`: JWT token for uploading guest programs to IPFS
- `PROGRAM_URL`: Current program at https://gateway.pinata.cloud/ipfs/QmajDapHhSM3Xm4aJAUk6BtgzqMsxTvpp3Y6abSAtkRKMV

## Development Patterns

### Guest Program Development
- Guest programs receive input via `env::stdin()` and decode using Alloy ABI encoding
- Use `env::commit_slice()` to commit the journal that contracts will verify
- Keep guest logic simple and deterministic
- For ML/AI programs: Use integer arithmetic to avoid floating-point operations in zkVM
- ETH prices should be handled in wei (18 decimals) for precision

### Contract Integration
- Contracts verify proofs using `VERIFIER.verify(seal, IMAGE_ID, journal_hash)`
- The journal must match expected data format between guest and contract
- Use `RiscZeroMockVerifier` for testing without generating real proofs
- Trading signals use tuple format: `(action: u8, confidence: u256, predicted_price: u256)`

### Client Application Patterns
- Use Boundless Client SDK for market interactions
- Submit requests onchain via `client.submit_onchain(request)`
- Wait for fulfillment before using proofs in contract calls
- Handle timeouts appropriately for proof generation
- Single-purpose application focused on trading signal generation

### AI/ML in zkVM Constraints
- No floating-point arithmetic - use integers and fixed-point math
- Embedded data for deterministic computation (no external oracles)
- Linear regression uses least squares with integer calculations
- Confidence scores as percentages (0-100) for readability

## File Structure Notes

- `contracts/src/`: Solidity source files
  - `TradingSignal.sol`: Main contract for AI trading signals
  - `ITradingSignal.sol`: Contract interface
  - `ImageID.sol`: Auto-generated guest program image IDs
- `contracts/scripts/`: Deployment scripts for TradingSignal contract
- `contracts/test/`: Foundry tests for TradingSignal functionality
- `guests/src/`: Shared guest utilities
- `guests/trading-signal/src/main.rs`: AI trading signal guest program
- `guests/tests/`: Guest program unit tests
- `apps/src/main.rs`: Client application for trading signal generation
- `foundry.toml`: Foundry configuration with custom paths
- `rust-toolchain.toml`: Pins Rust version to 1.89 for RISC Zero compatibility

## AI/ML Trading Signal Features

The `trading-signal` guest implements a simple linear regression model for ETH price prediction:

- **Input**: Current ETH price in wei
- **Algorithm**: Linear regression on 30 days of embedded historical price data
- **Output**: Trading action (0=SELL, 1=BUY), confidence score (0-100%), predicted price in wei
- **Decision Logic**: BUY if predicted price > current price + 0.5% threshold
- **Data Format**: All prices in wei (18 decimals) for precision without floating-point

## Common Issues and Solutions

### RISC Zero Binary Format Issue
**Problem**: "Malformed ProgramBinary" error when using Boundless Market
**Root Cause**: RISC Zero guest programs generate both `.elf` and `.bin` files. The Boundless Market expects the `.bin` format (raw binary), not the `.elf` format.
**Solution**: 
- Use `target/riscv-guest/guests/{program}/riscv32im-risc0-zkvm-elf/release/{program}.bin` instead of the `.elf` file
- When uploading to IPFS, use the `.bin` file
- This applies to both local execution and URL-based program distribution

### Base Mainnet Deployment Configuration
**Problem**: Chain ID mismatch errors when connecting to Base mainnet
**Root Cause**: Boundless SDK auto-resolves deployment config but may default to wrong network
**Solution**: Explicitly specify deployment parameters:
```bash
--chain-id 8453 --boundless-market-address 0xfd152dadc5183870710fe54f939eae3ab9f0fe82 --set-verifier-address 0x1Ab08498CfF17b9723ED67143A050c8E8c2e3104
```

### ETH Precision and Overflow
**Problem**: u64 overflow when handling ETH prices in wei
**Root Cause**: ETH prices like 3700 ETH = 3,700,000,000,000,000,000,000 wei exceed u64 max
**Solution**: Use gwei scale (10^9) internally, convert to wei for external interfaces

### IMAGE_ID Mismatch and Proof Verification Errors
**Problem**: Error `0x439cc0cd` when calling `setSignal` on the contract
**Root Cause**: The guest program IMAGE_ID doesn't match what's expected by the contract
**Solution**: 
1. **Contract has dynamic IMAGE_ID**: Use `setImageId(bytes32)` to update the contract
2. **Check current IMAGE_ID**: Run `cargo build` to see the generated `ImageID.sol`
3. **Update Pinata binary**: Upload new `.bin` file if guest program changed
4. **Boundless cache**: May need to wait for Boundless to pick up new binary

**Current Working Configuration**:
- Contract IMAGE_ID: `0x84350b9c8ced73a096f4531bb7ab340be0eca8ab216a8cae51917a3b40fdc659` (Boundless cached)
- Guest program should commit tuple: `(U256::from(signal), U256::from(confidence), U256::from(predicted_price))`

### Pinata Binary Upload
```bash
# Upload new guest binary to Pinata
curl -X POST "https://api.pinata.cloud/pinning/pinFileToIPFS" \
  -H "Authorization: Bearer $PINATA_JWT" \
  -F file=@"target/riscv-guest/guests/trading-signal/riscv32im-risc0-zkvm-elf/release/trading-signal.bin"
```

## Rust Toolchain

This project uses Rust 1.89 as specified in `rust-toolchain.toml` for RISC Zero compatibility. The toolchain includes clippy, rustfmt, and rust-src components.

## Trading Signal Algorithm

The AI uses linear regression on 30 days of embedded ETH price history:
- **Input**: Current ETH amount in wei (e.g., 3.7 ETH = 3700000000000000000 wei)
- **Processing**: Assumes current USD price ($3200/ETH), runs linear regression to predict next day
- **Output**: BUY/SELL signal with confidence % and predicted price
- **Current Performance**: 97% confidence BUY signal, predicting 1.10 ETH (~$3520/ETH)
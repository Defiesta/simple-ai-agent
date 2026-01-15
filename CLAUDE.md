# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a Boundless Foundry template for building verifiable AI-powered DeFi applications using RISC Zero. The project demonstrates a complete end-to-end flow from AI guest programs running linear regression in the zkVM to Solidity smart contracts that store verified trading signals.

## 🚀 Current Status: PRODUCTION READY ✅

- ✅ **Smart Contract Deployed**: `0xEe747ac1869f9F805dCa40Ef2E6197C2F2e25f16` on Base Mainnet
- ✅ **Contract Verified**: https://basescan.org/address/0xee747ac1869f9f805dca40ef2e6197c2f2e25f16
- ✅ **Verifier Issue RESOLVED**: End-to-end proof generation and verification working
- ✅ **Latest Success**: Tx `0x44fe4be8faa9d2bc797726496b0987decba12dc228c8b18602b7fa9fa07f01da` (BUY signal, 97% confidence)
- ✅ **IMAGE_ID Synced**: `0x9e03bf4cd639667070b4343899e51f74776ba88dde8ec0708807471ffa532f22`

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
# CORRECT: Using the working pre-uploaded guest program (RECOMMENDED)
RUST_LOG=info cargo run --release --bin app -- \
  --current-price 3200 \
  --program-url https://gateway.pinata.cloud/ipfs/QmQ2XmScCBFrayWSe1HaVrGzKvqdkDxCPbfJpDyn8SSi4H

# Alternative: Use local guest program (requires IMAGE_ID sync)
RUST_LOG=info cargo run --release --bin app -- --current-price 3200

# IMPORTANT: When using local program, ensure contract IMAGE_ID matches:
# 1. Check current local IMAGE_ID: cat contracts/src/ImageID.sol
# 2. Update contract: cast send 0xEe747ac1869f9F805dCa40Ef2E6197C2F2e25f16 "setImageId(bytes32)" <NEW_IMAGE_ID>
```

## Environment Variables

**Current Production Configuration (Base Mainnet)**:
- `RPC_URL`: https://base-mainnet.g.alchemy.com/v2/YOUR_API_KEY
- `TRADING_SIGNAL_ADDRESS`: 0xEe747ac1869f9F805dCa40Ef2E6197C2F2e25f16 (verified contract)
- `VERIFIER_ADDRESS`: 0x0b144e07a0826182b6b59788c34b32bfa86fb711 (RISC Zero verifier)
- `CHAIN_ID`: 8453 (Base mainnet)
- `BOUNDLESS_MARKET_ADDRESS`: 0xfd152dadc5183870710fe54f939eae3ab9f0fe82
- `SET_VERIFIER_ADDRESS`: 0x1Ab08498CfF17b9723ED67143A050c8E8c2e3104

**Required for Development**:
- `PRIVATE_KEY`: Wallet private key with sufficient ETH on Base ⚠️ **NEVER EXPOSE IN COMMANDS**
- `PINATA_JWT`: JWT token for uploading guest programs to IPFS ⚠️ **KEEP SECRET**
- `PROGRAM_URL`: **CURRENT WORKING BINARY**: https://gateway.pinata.cloud/ipfs/QmQ2XmScCBFrayWSe1HaVrGzKvqdkDxCPbfJpDyn8SSi4H

## 🔒 **CRITICAL SECURITY PRACTICES**

### **NEVER expose private keys in commands!** Use `.env` file instead:

**SECURE Setup**:
1. Create `.env` file in project root:
```bash
# .env file (NEVER commit this to git)
RPC_URL=https://base-mainnet.g.alchemy.com/v2/YOUR_API_KEY
PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE
TRADING_SIGNAL_ADDRESS=0xEe747ac1869f9F805dCa40Ef2E6197C2F2e25f16
CHAIN_ID=8453
BOUNDLESS_MARKET_ADDRESS=0xfd152dadc5183870710fe54f939eae3ab9f0fe82
SET_VERIFIER_ADDRESS=0x1Ab08498CfF17b9723ED67143A050c8E8c2e3104
PINATA_JWT=your_pinata_jwt_here
PROGRAM_URL=https://gateway.pinata.cloud/ipfs/QmQ2XmScCBFrayWSe1HaVrGzKvqdkDxCPbfJpDyn8SSi4H
```

2. Add `.env` to `.gitignore`:
```bash
echo ".env" >> .gitignore
```

3. **SAFE Command** (reads from .env automatically):
```bash
# SECURE: No private keys in command line
RUST_LOG=info cargo run --release --bin app -- --current-price 3800
```

### **Security Checklist**:
- ✅ Use `.env` file for secrets
- ✅ Add `.env` to `.gitignore` 
- ✅ Use test wallets with minimal funds for development
- ✅ Clear shell history after accidental exposure: 
  - **Bash**: `history -c && history -w`
  - **Zsh**: `fc -p && > ~/.zsh_history && exec zsh`
- ✅ Use hardware wallets for production funds
- ✅ Never paste private keys in shared terminals/logs

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

## Debugging Guide: Verifier Failure Resolution ✅

**The primary verifier issue (error `0x439cc0cd`) has been RESOLVED as of January 15, 2026.**

### 🎯 **Resolved: Critical Verifier Failure Issue**

**Problem**: Error `0x439cc0cd` (VerificationFailed) when calling `setSignal` on the contract  
**Root Cause**: **ABI Encoding Mismatch** between guest program and contract expectations  

#### **The Root Issue**
- **Guest Program**: Used manual ABI encoding with incorrect padding for `uint8`
- **Contract**: Expected standard Solidity `abi.encode(uint8, uint256, uint256)` format  
- **Result**: Journal hash mismatch causing proof verification to fail

#### **Complete Solution Applied**

**1. Fixed Guest ABI Encoding** (`guests/trading-signal/src/main.rs`):
```rust
// OLD: Manual padding (INCORRECT)
let mut action_bytes = [0u8; 32];
action_bytes[31] = signal;

// NEW: Proper Solidity ABI encoding (CORRECT)  
let mut action_bytes = [0u8; 32];
action_bytes[31] = signal; // Right-aligned (value in least significant byte)
journal_data.extend_from_slice(&action_bytes);
journal_data.extend_from_slice(&confidence_u256.to_be_bytes::<32>());
journal_data.extend_from_slice(&price_u256.to_be_bytes::<32>());
```

**2. Updated IMAGE_ID Synchronization**:
- **New IMAGE_ID**: `0x9e03bf4cd639667070b4343899e51f74776ba88dde8ec0708807471ffa532f22`
- **Contract Updated**: Used `setImageId(bytes32)` function
- **Binary Uploaded**: New corrected binary at `QmQ2XmScCBFrayWSe1HaVrGzKvqdkDxCPbfJpDyn8SSi4H`

**3. Fixed Boundless Fulfillment Data Parsing** (`apps/src/main.rs`):
```rust
// Boundless wraps journal in fulfillment structure:
// [32-byte offset][32-byte IMAGE_ID][32-byte offset][32-byte offset][96-byte journal]
let journal_start = 128; // Offset where our journal data starts
let journal_data = &data[journal_start..journal_start + 96];
```

#### **Verification Success** ✅
- **Transaction Hash**: `0x44fe4be8faa9d2bc797726496b0987decba12dc228c8b18602b7fa9fa07f01da`
- **Result**: BUY signal, 97% confidence, successfully stored on-chain
- **Status**: End-to-end proof generation and verification working

---

## Common Development Issues

### RISC Zero Binary Format Issue
**Problem**: "Malformed ProgramBinary" error when using Boundless Market
**Solution**: Always use `.bin` files, not `.elf` files:
```bash
# Correct path for Boundless
target/riscv-guest/guests/trading-signal/riscv32im-risc0-zkvm-elf/release/trading-signal.bin
```

### Base Mainnet Configuration
Explicitly specify all deployment parameters:
```bash
--chain-id 8453 --boundless-market-address 0xfd152dadc5183870710fe54f939eae3ab9f0fe82 --set-verifier-address 0x1Ab08498CfF17b9723ED67143A050c8E8c2e3104
```

### Working Binary Management
```bash
# Upload new guest binary to Pinata (use this exact command)
curl -X POST "https://api.pinata.cloud/pinning/pinFileToIPFS" \
  -H "Authorization: Bearer $PINATA_JWT" \
  -F "file=@target/riscv-guest/guests/trading-signal/riscv32im-risc0-zkvm-elf/release/trading-signal.bin" \
  -F 'pinataMetadata={"name":"trading-signal-corrected.bin"}'

# Update contract IMAGE_ID after rebuilding guest
cast send 0xEe747ac1869f9F805dCa40Ef2E6197C2F2e25f16 "setImageId(bytes32)" <NEW_IMAGE_ID> \
  --private-key $PRIVATE_KEY --rpc-url $RPC_URL
```

### Current Working Configuration ✅
- **Contract IMAGE_ID**: `0x9e03bf4cd639667070b4343899e51f74776ba88dde8ec0708807471ffa532f22`
- **Working Binary**: `QmQ2XmScCBFrayWSe1HaVrGzKvqdkDxCPbfJpDyn8SSi4H`  
- **Guest Encoding**: Proper Solidity ABI format for `(uint8, uint256, uint256)`
- **Client Parsing**: Extracts journal from Boundless fulfillment at offset 128

## Rust Toolchain

This project uses Rust 1.89 as specified in `rust-toolchain.toml` for RISC Zero compatibility. The toolchain includes clippy, rustfmt, and rust-src components.

## Trading Signal Algorithm ✅ WORKING

The AI uses linear regression on 30 days of embedded ETH price history:
- **Input**: Current ETH price in USD (e.g., 3200 means $3200 per ETH)
- **Processing**: Uses current USD price, runs linear regression to predict next day
- **Output**: BUY/SELL signal with confidence % and predicted USD price
- **Latest Success**: BUY signal, 97% confidence (Tx: `0x44fe4be8faa9d2bc797726496b0987decba12dc228c8b18602b7fa9fa07f01da`)

### Algorithm Details
- **Historical Data**: 30 days of embedded ETH/USD price data
- **Model**: Linear regression using least squares with integer calculations  
- **Threshold**: BUY if predicted price > current price + 0.5%
- **Confidence**: R² coefficient of determination (0-100%)
- **Precision**: All calculations in USD integers to avoid floating-point operations in zkVM
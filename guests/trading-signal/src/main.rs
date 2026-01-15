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

use std::io::Read;

use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use risc0_zkvm::guest::env;

// Historical ETH price data (30 days) - USD price per ETH
// Format: (day_index, usd_price_per_eth)
// These are actual USD prices, e.g., 3200 means $3200 per ETH
const PRICE_HISTORY: [(u64, u64); 30] = [
    (1, 3200),   // Day 1: $3200 per ETH
    (2, 3215),   // Day 2: $3215 per ETH  
    (3, 3189),   // Day 3: $3189 per ETH
    (4, 3221),   // Day 4: $3221 per ETH
    (5, 3254),   // Day 5: $3254 per ETH
    (6, 3278),   // Day 6: $3278 per ETH
    (7, 3242),   // Day 7: $3242 per ETH
    (8, 3291),   // Day 8: $3291 per ETH
    (9, 3315),   // Day 9: $3315 per ETH
    (10, 3287),  // Day 10: $3287 per ETH
    (11, 3324),  // Day 11: $3324 per ETH
    (12, 3352),  // Day 12: $3352 per ETH
    (13, 3389),  // Day 13: $3389 per ETH
    (14, 3412),  // Day 14: $3412 per ETH
    (15, 3398),  // Day 15: $3398 per ETH
    (16, 3436),  // Day 16: $3436 per ETH
    (17, 3462),  // Day 17: $3462 per ETH
    (18, 3489),  // Day 18: $3489 per ETH
    (19, 3453),  // Day 19: $3453 per ETH
    (20, 3507),  // Day 20: $3507 per ETH
    (21, 3534),  // Day 21: $3534 per ETH
    (22, 3561),  // Day 22: $3561 per ETH
    (23, 3528),  // Day 23: $3528 per ETH
    (24, 3582),  // Day 24: $3582 per ETH
    (25, 3615),  // Day 25: $3615 per ETH
    (26, 3648),  // Day 26: $3648 per ETH
    (27, 3621),  // Day 27: $3621 per ETH
    (28, 3674),  // Day 28: $3674 per ETH
    (29, 3702),  // Day 29: $3702 per ETH
    (30, 3735),  // Day 30: $3735 per ETH
];

fn linear_regression() -> (i64, i64, u64) {
    let n = PRICE_HISTORY.len() as i64;
    
    // Calculate means
    let sum_x: i64 = PRICE_HISTORY.iter().map(|(x, _)| *x as i64).sum();
    let sum_y: i64 = PRICE_HISTORY.iter().map(|(_, y)| *y as i64).sum();
    let mean_x = sum_x / n;
    let mean_y = sum_y / n;
    
    // Calculate slope (m) and intercept (b)
    let mut numerator = 0i64;
    let mut denominator = 0i64;
    let mut sum_squared_errors = 0i64;
    let mut sum_squared_total = 0i64;
    
    for (x, y) in PRICE_HISTORY.iter() {
        let x_diff = *x as i64 - mean_x;
        let y_diff = *y as i64 - mean_y;
        
        numerator += x_diff * y_diff;
        denominator += x_diff * x_diff;
        sum_squared_total += y_diff * y_diff;
    }
    
    let slope = if denominator != 0 { numerator / denominator } else { 0 };
    let intercept = mean_y - slope * mean_x;
    
    // Calculate R² for confidence (coefficient of determination)
    for (x, y) in PRICE_HISTORY.iter() {
        let predicted = slope * (*x as i64) + intercept;
        let error = *y as i64 - predicted;
        sum_squared_errors += error * error;
    }
    
    let r_squared = if sum_squared_total > 0 {
        let ratio = (sum_squared_total - sum_squared_errors) * 100 / sum_squared_total;
        if ratio > 0 { ratio as u64 } else { 0 }
    } else {
        0
    };
    
    (slope, intercept, r_squared.min(100))
}

fn main() {
    // Read the input data - this is the amount of wei that represents the current USD value
    // For example: if ETH price is $3200, then 3700000000000000000 wei = 3.7 ETH = $11,840 worth
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();
    let current_eth_amount = <U256>::abi_decode(&input_bytes).unwrap();
    let current_eth_amount_wei = current_eth_amount.as_limbs()[0];
    
    // We need to assume a current USD price per ETH to make sense of the input
    // Let's assume current ETH price is $3200 (around the average of our historical data)
    let assumed_current_usd_price_per_eth = 3200u64;
    
    // Perform linear regression on USD prices
    let (slope, intercept, confidence) = linear_regression();
    
    // Predict next day USD price (day 31)
    let next_day = 31i64;
    let predicted_usd_price_per_eth = (slope * next_day + intercept) as u64;
    
    // Convert predicted USD price back to wei equivalent
    // If predicted price is $3400 per ETH, and we have 3.7 ETH worth in wei,
    // then predicted value = (3400/3200) * current_eth_amount_wei
    let predicted_price_wei = if assumed_current_usd_price_per_eth > 0 {
        (current_eth_amount_wei * predicted_usd_price_per_eth) / assumed_current_usd_price_per_eth
    } else {
        current_eth_amount_wei
    };
    
    // Generate trading signal
    // BUY (1) if predicted USD price is > 0.5% higher than current USD price
    // SELL (0) otherwise
    let price_threshold = assumed_current_usd_price_per_eth + (assumed_current_usd_price_per_eth / 200); // 0.5% increase
    let signal = if predicted_usd_price_per_eth > price_threshold { 1u8 } else { 0u8 };
    
    // Create the exact same journal format as the contract expects: abi.encode(uint8, uint256, uint256)  
    // Prepare the values - use proper Solidity ABI encoding matching exactly what the contract test does
    let confidence_u256 = U256::from(confidence);
    let price_u256 = U256::from(predicted_price_wei);
    
    // Use manual encoding that exactly matches Solidity's abi.encode for (uint8, uint256, uint256)
    let mut journal_data = Vec::new();
    
    // For Solidity abi.encode, uint8 is right-aligned in 32 bytes (big-endian padding)
    let mut action_bytes = [0u8; 32];
    action_bytes[31] = signal; // Right-aligned (value in least significant byte)
    journal_data.extend_from_slice(&action_bytes);
    
    // U256 values are encoded as 32-byte big-endian
    journal_data.extend_from_slice(&confidence_u256.to_be_bytes::<32>());
    journal_data.extend_from_slice(&price_u256.to_be_bytes::<32>());
    
    env::commit_slice(&journal_data);
}
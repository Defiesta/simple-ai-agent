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

pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {Receipt as RiscZeroReceipt} from "risc0/IRiscZeroVerifier.sol";
import {RiscZeroMockVerifier} from "risc0/test/RiscZeroMockVerifier.sol";
import {VerificationFailed} from "risc0/IRiscZeroVerifier.sol";
import {TradingSignal} from "../src/TradingSignal.sol";
import {ITradingSignal} from "../src/ITradingSignal.sol";
import {ImageID} from "../src/ImageID.sol";

contract TradingSignalTest is RiscZeroCheats, Test {
    TradingSignal public tradingSignal;
    RiscZeroMockVerifier public verifier;

    function setUp() public {
        verifier = new RiscZeroMockVerifier(0);
        tradingSignal = new TradingSignal(verifier);
        
        // Check initial state
        ITradingSignal.Signal memory initial = tradingSignal.getLatestSignal();
        assertEq(initial.action, 0);
        assertEq(initial.confidence, 0);
        assertEq(initial.predictedPrice, 0);
    }

    function test_SetBuySignal() public {
        uint8 action = 1; // BUY
        uint256 confidence = 85;
        uint256 predictedPrice = 3750000000000000000; // 3.75 ETH in wei

        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action, confidence, predictedPrice))
        );

        tradingSignal.setSignal(action, confidence, predictedPrice, receipt.seal);
        
        ITradingSignal.Signal memory signal = tradingSignal.getLatestSignal();
        assertEq(signal.action, 1);
        assertEq(signal.confidence, 85);
        assertEq(signal.predictedPrice, 3750000000000000000);
        assertEq(tradingSignal.shouldBuy(), true);
        assertEq(tradingSignal.shouldSell(), false);
        assertEq(tradingSignal.getActionString(), "BUY");
    }

    function test_SetSellSignal() public {
        uint8 action = 0; // SELL
        uint256 confidence = 92;
        uint256 predictedPrice = 3400000000000000000; // 3.4 ETH in wei

        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action, confidence, predictedPrice))
        );

        tradingSignal.setSignal(action, confidence, predictedPrice, receipt.seal);
        
        ITradingSignal.Signal memory signal = tradingSignal.getLatestSignal();
        assertEq(signal.action, 0);
        assertEq(signal.confidence, 92);
        assertEq(signal.predictedPrice, 3400000000000000000);
        assertEq(tradingSignal.shouldBuy(), false);
        assertEq(tradingSignal.shouldSell(), true);
        assertEq(tradingSignal.getActionString(), "SELL");
    }

    function test_RejectInvalidAction() public {
        uint8 invalidAction = 2; // Invalid
        uint256 confidence = 80;
        uint256 predictedPrice = 3600000000000000000;

        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(invalidAction, confidence, predictedPrice))
        );

        vm.expectRevert("Invalid action: must be 0 (SELL) or 1 (BUY)");
        tradingSignal.setSignal(invalidAction, confidence, predictedPrice, receipt.seal);
    }

    function test_RejectInvalidConfidence() public {
        uint8 action = 1;
        uint256 invalidConfidence = 101; // > 100
        uint256 predictedPrice = 3600000000000000000;

        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action, invalidConfidence, predictedPrice))
        );

        vm.expectRevert("Invalid confidence: must be 0-100");
        tradingSignal.setSignal(action, invalidConfidence, predictedPrice, receipt.seal);
    }

    function test_RejectZeroPredictedPrice() public {
        uint8 action = 1;
        uint256 confidence = 80;
        uint256 invalidPrice = 0;

        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action, confidence, invalidPrice))
        );

        vm.expectRevert("Invalid predicted price: must be > 0");
        tradingSignal.setSignal(action, confidence, invalidPrice, receipt.seal);
    }

    function test_RejectInvalidProof() public {
        // Create a proof for different data than what we're submitting
        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(1, 80, 350000))
        );

        // Try to submit different data with the wrong proof
        vm.expectRevert(VerificationFailed.selector);
        tradingSignal.setSignal(0, 90, 340000, receipt.seal);
    }

    function test_EventEmission() public {
        uint8 action = 1;
        uint256 confidence = 75;
        uint256 predictedPrice = 3650000000000000000;

        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action, confidence, predictedPrice))
        );

        vm.expectEmit(true, false, false, false);
        emit ITradingSignal.SignalUpdated(action, confidence, predictedPrice, block.timestamp);
        
        tradingSignal.setSignal(action, confidence, predictedPrice, receipt.seal);
    }

    function test_MultiplUpdates() public {
        // First signal: BUY
        uint8 action1 = 1;
        uint256 confidence1 = 80;
        uint256 price1 = 3700000000000000000;

        RiscZeroReceipt memory receipt1 = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action1, confidence1, price1))
        );

        tradingSignal.setSignal(action1, confidence1, price1, receipt1.seal);
        assertEq(tradingSignal.getSignalAction(), 1);

        // Second signal: SELL
        uint8 action2 = 0;
        uint256 confidence2 = 95;
        uint256 price2 = 3500000000000000000;

        RiscZeroReceipt memory receipt2 = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action2, confidence2, price2))
        );

        tradingSignal.setSignal(action2, confidence2, price2, receipt2.seal);
        assertEq(tradingSignal.getSignalAction(), 0);
        assertEq(tradingSignal.getConfidence(), 95);
        assertEq(tradingSignal.getPredictedPrice(), 3500000000000000000);
    }

    function test_ViewFunctions() public {
        uint8 action = 1;
        uint256 confidence = 88;
        uint256 predictedPrice = 3800000000000000000;

        RiscZeroReceipt memory receipt = verifier.mockProve(
            ImageID.TRADING_SIGNAL_ID, 
            sha256(abi.encode(action, confidence, predictedPrice))
        );

        tradingSignal.setSignal(action, confidence, predictedPrice, receipt.seal);

        // Test individual getters
        assertEq(tradingSignal.getSignalAction(), 1);
        assertEq(tradingSignal.getConfidence(), 88);
        assertEq(tradingSignal.getPredictedPrice(), 3800000000000000000);
        
        // Test complete signal getter
        ITradingSignal.Signal memory signal = tradingSignal.getLatestSignal();
        assertEq(signal.action, 1);
        assertEq(signal.confidence, 88);
        assertEq(signal.predictedPrice, 3800000000000000000);
        assertTrue(signal.timestamp > 0);
    }
}
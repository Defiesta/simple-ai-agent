// Copyright 2025 RISC Zero, Inc.
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

import {Script, console2} from "forge-std/Script.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {TradingSignal} from "../src/TradingSignal.sol";

contract Deploy is Script {
    function run() external {
        // load ENV variables first
        uint256 key = vm.envUint("PRIVATE_KEY");
        address verifierAddress = vm.envAddress("VERIFIER_ADDRESS");
        
        vm.startBroadcast(key);

        IRiscZeroVerifier verifier = IRiscZeroVerifier(verifierAddress);
        TradingSignal tradingSignal = new TradingSignal(verifier);
        address tradingSignalAddress = address(tradingSignal);
        console2.log("Deployed TradingSignal to", tradingSignalAddress);
        console2.log("Set environment variable: export TRADING_SIGNAL_ADDRESS=", tradingSignalAddress);

        vm.stopBroadcast();
        
        // Verify the contract on Etherscan/Basescan
        vm.startPrank(msg.sender);
        string memory apiKey = vm.envString("ETHERSCAN_API_KEY");
        if (bytes(apiKey).length > 0) {
            console2.log("Verifying contract on Etherscan...");
            string[] memory cmd = new string[](10);
            cmd[0] = "forge";
            cmd[1] = "verify-contract";
            cmd[2] = "--chain-id";
            cmd[3] = vm.toString(block.chainid);
            cmd[4] = "--constructor-args";
            cmd[5] = vm.toString(abi.encode(verifierAddress));
            cmd[6] = "--etherscan-api-key";
            cmd[7] = apiKey;
            cmd[8] = vm.toString(tradingSignalAddress);
            cmd[9] = "contracts/src/TradingSignal.sol:TradingSignal";
            
            bytes memory result = vm.ffi(cmd);
            console2.log("Verification result:", string(result));
        }
        vm.stopPrank();
    }
}

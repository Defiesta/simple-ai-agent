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

interface ITradingSignal {
    struct Signal {
        uint8 action;           // 0 = SELL, 1 = BUY
        uint256 confidence;     // Confidence score (0-100)
        uint256 predictedPrice; // Predicted price in wei (18 decimals)
        uint256 timestamp;      // When signal was generated
    }

    event SignalUpdated(
        uint8 indexed action,
        uint256 confidence,
        uint256 predictedPrice,
        uint256 timestamp
    );

    event ImageIdUpdated(bytes32 indexed imageId);

    function setSignal(
        uint8 action,
        uint256 confidence, 
        uint256 predictedPrice,
        bytes calldata seal
    ) external;

    function setImageId(bytes32 _imageId) external;

    function getLatestSignal() external view returns (Signal memory);
    
    function getSignalAction() external view returns (uint8);
    
    function getConfidence() external view returns (uint256);
    
    function getPredictedPrice() external view returns (uint256);
}
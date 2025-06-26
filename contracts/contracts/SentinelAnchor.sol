// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SentinelAnchor
/// @notice Emits immutable events linking a BLAKE-3 Merkle root to chain time.
/// @dev No storage – cheapest possible anchor.
contract SentinelAnchor {
    event RootAnchored(bytes32 indexed root, uint256 timestamp);

    function anchor(bytes32 root) external {
        emit RootAnchored(root, block.timestamp);
    }
}

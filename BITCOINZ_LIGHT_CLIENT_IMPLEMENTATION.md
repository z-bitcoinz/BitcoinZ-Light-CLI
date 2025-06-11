# BitcoinZ Light Client Implementation

## Overview

This document describes the implementation of BitcoinZ support in the Zcash light client. The BitcoinZ light client enables users to send and receive BitcoinZ (BTCZ) transactions without running a full node, using the lightwalletd protocol.

## Key Achievements

- ✅ Successfully implemented transparent-to-transparent (t→t) transactions
- ✅ Proper v4 (Sapling) transaction format support for BitcoinZ
- ✅ Correct sighash computation for BitcoinZ's consensus rules
- ✅ Integration with existing Zcash light client infrastructure

## Architecture

The BitcoinZ implementation extends the existing Zcash light client with BitcoinZ-specific modules:

```
lib/src/
├── lib.rs                           # BitcoinZ network definition
├── bitcoinz_v4_no_sig.rs           # V4 transaction builder
├── bitcoinz_transaction.rs         # Transaction type detection
├── bitcoinz_branch.rs              # Branch ID management
├── bitcoinz_binding_sig_fix.rs     # Binding signature analysis
└── lightwallet.rs                  # Integration point
```

## Key Differences from Zcash

### 1. Network Parameters
- **Coin Type**: 177 (BitcoinZ)
- **P2PKH Prefix**: `[0x1c, 0xb8]`
- **P2SH Prefix**: `[0x1c, 0xbd]`
- **Sapling HRP**: "zs" (same as Zcash)

### 2. Activation Heights
- **Overwinter**: Block 328,500
- **Sapling**: Block 328,500 (simultaneous activation)
- **Blossom**: Block 653,600
- **Heartwood**: Block 903,800
- **Canopy**: Block 1,153,550
- **Nu5**: Not yet supported

### 3. Transaction Format
- **Version**: 4 (Sapling)
- **Version Group ID**: `0x892f2085`
- **Expiry Height**: 0 (no expiry by default)
- **Binding Signature**: Not required for transparent-only transactions

### 4. Sighash Computation
The most critical difference is in sighash computation:
- BitcoinZ uses BLAKE2b with Sapling personalization
- Consensus Branch ID: `1991772603` (0x76b809bb)
- **IMPORTANT**: The sighash bytes are NOT reversed after BLAKE2b computation

## Usage

### Building the Client

```bash
cd /path/to/btcz-light-cli
cargo build --release
```

### Running the Client

```bash
# Connect to BitcoinZ lightwalletd server
./target/release/bitcoinz-light-cli --server http://btcz-lightwalletd:9067

# Import wallet
./bitcoinz-light-cli import "your seed phrase here"

# Check balance
./bitcoinz-light-cli balance

# Send transaction
./bitcoinz-light-cli send "t1AddressHere" 0.1
```

### Configuration

The client can be configured with:
- `--server`: BitcoinZ lightwalletd server URL
- `--data-dir`: Directory for wallet data
- `--chain`: Network (mainnet/testnet)

## Implementation Details

### Transaction Builder

The `bitcoinz_v4_no_sig` module implements a custom transaction builder that:
1. Creates v4 Sapling format transactions
2. Computes sighashes using BitcoinZ's algorithm
3. Handles transparent inputs and outputs
4. Omits binding signature for transparent-only transactions

### Sighash Algorithm

```rust
// Key components of BitcoinZ sighash computation
const CONSENSUS_BRANCH_ID: u32 = 1991772603; // 0x76b809bb
const VERSION_GROUP_ID: u32 = 0x892f2085;

// Personalization for BLAKE2b
let mut personalization = [0u8; 16];
personalization[..12].copy_from_slice(b"ZcashSigHash");
personalization[12..16].copy_from_slice(&CONSENSUS_BRANCH_ID.to_le_bytes());

// Compute BLAKE2b hash
let hash = blake2b(data, personalization);
// DO NOT reverse the hash bytes!
```

### Integration Points

The light wallet integrates BitcoinZ support through:
1. `detect_tx_type()` - Identifies transaction type
2. `should_use_bitcoinz_builder()` - Determines when to use custom builder
3. `build_bitcoinz_v4_no_sig()` - Creates BitcoinZ-compatible transactions

## Troubleshooting

### Common Issues

1. **"bad-txns-sapling-binding-signature-invalid"**
   - Cause: Using standard Zcash transaction format
   - Solution: Ensure BitcoinZ v4 builder is used for transparent transactions

2. **"mandatory-script-verify-flag-failed"**
   - Cause: Incorrect sighash computation
   - Solution: Verify sighash is not reversed after BLAKE2b

3. **"tx-overwinter-active"**
   - Cause: Attempting to use pre-Overwinter transaction format
   - Solution: Use v4 transaction format

### Debug Mode

Enable debug output by setting:
```bash
export RUST_LOG=debug
```

## Security Considerations

1. **Private Keys**: Never exposed or logged
2. **Seed Phrases**: Stored encrypted in wallet file
3. **Network Communication**: Use HTTPS for lightwalletd connections
4. **Transaction Signing**: All signing done locally

## Future Work

### Phase 2: Shielded Transactions
- Implement z→t transactions
- Implement t→z transactions
- Implement z→z transactions
- Fix binding signature computation for shielded components

### Improvements
- Add hardware wallet support
- Implement view-only wallets
- Add multi-signature support
- Optimize transaction building performance

## Contributing

When contributing to the BitcoinZ light client:
1. Follow Rust coding conventions
2. Add tests for new functionality
3. Update documentation
4. Test on both mainnet and testnet

## License

This implementation follows the license of the parent Zcash light client project.

## Acknowledgments

- Zcash developers for the light client foundation
- BitcoinZ community for protocol specifications
- Contributors to the librustzcash library
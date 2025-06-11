# Add BitcoinZ transparent transaction support

## Summary
This PR implements BitcoinZ v4 transaction builder for transparent-to-transparent (t→t) transfers, enabling the light client to send BitcoinZ transactions without running a full node.

## Key Features
- ✅ Custom v4 transaction builder without binding signatures for t→t transactions
- ✅ BitcoinZ-specific sighash computation (no byte reversal after BLAKE2b)
- ✅ Complete BitcoinZ network parameters implementation
- ✅ Extensive documentation including implementation guide, transaction specification, and testing guide

## Technical Details
- **Consensus Branch ID**: `1991772603` (0x76b809bb) - Fixed for all heights
- **Version Group ID**: `0x892f2085` - BitcoinZ Sapling
- **Transaction Version**: 4 (Sapling with Overwinter flag)
- **Expiry Height**: 0 (no expiry by default)
- **Key Discovery**: BitcoinZ doesn't reverse sighash bytes after BLAKE2b computation

## Testing
Successfully created and broadcast transaction on mainnet:
- **Transaction ID**: `7ca9d30f60cf1e9d3be0cfefbaeccd713cf04b631722d5882e6ce9373cde6065`
- **Type**: Transparent-to-Transparent (t→t)
- **Amount**: 0.1 BTCZ
- **Status**: Confirmed on BitcoinZ network

## Documentation Added
- `BITCOINZ_LIGHT_CLIENT_IMPLEMENTATION.md` - Comprehensive implementation guide
- `BITCOINZ_TRANSACTION_SPECIFICATION.md` - Technical transaction format details
- `BITCOINZ_TESTING_GUIDE.md` - Step-by-step testing procedures
- `BITCOINZ_IMPLEMENTATION_STATUS.md` - Current status and progress tracking

## Code Structure
```
lib/src/
├── bitcoinz_v4_no_sig.rs           # Core v4 transaction builder
├── bitcoinz_transaction.rs         # Transaction type detection
├── bitcoinz_branch.rs              # Branch ID management
├── bitcoinz_binding_sig*.rs        # Binding signature analysis
├── lib.rs                          # BitcoinZ network definition
└── lightwallet.rs                  # Integration point
```

## Known Issues
- Build warnings for unused imports (can be cleaned up in follow-up PR)
- CI build failures need investigation (builds successfully locally)

## Future Work (Phase 2)
- Implement shielded transactions (t→z, z→t, z→z)
- Add hardware wallet support
- Performance optimizations

## How to Test
```bash
# Build
cargo build --release

# Import wallet
./target/release/bitcoinz-light-cli --server http://93.107.37.216:9067 import "seed phrase"

# Send transaction
./target/release/bitcoinz-light-cli --server http://93.107.37.216:9067 send "t1Address" 0.1
```

## Breaking Changes
None - This PR only adds BitcoinZ support without affecting existing Zcash functionality.
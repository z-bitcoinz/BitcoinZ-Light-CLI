# BitcoinZ Light Client Implementation Status

## Phase 1: Transparent-to-Transparent (t→t) Transactions ✅ COMPLETE

### Completed
1. **BitcoinZ Network Implementation**
   - Created `BitcoinZMainNetwork` with correct activation heights
   - Overwinter: block 328,500
   - Sapling: block 328,500
   - Blossom: block 653,600
   - Heartwood: block 903,800
   - Canopy: block 1,153,550
   - Nu5: Not supported yet

2. **Transaction Detection Module** (`bitcoinz_transaction.rs`)
   - Detects transaction types: t→t, t→z, z→t, z→z
   - Used to apply appropriate fixes based on transaction type

3. **Binding Signature Analysis** (`bitcoinz_binding_sig.rs`, `bitcoinz_binding_sig_fix.rs`)
   - Analyzes why transactions fail
   - Discovered BitcoinZ validates binding signatures differently than Zcash
   - BitcoinZ: `sign(bsk, bvk || sighash)` [64 bytes]
   - Zcash: `sign(bsk, sighash)` [32 bytes]

4. **Overwinter Transaction Builder** (`bitcoinz_overwinter_builder.rs`)
   - Placeholder implementation that falls back to standard builder
   - Would build v3 transactions to bypass Sapling binding signature
   - Requires manual transaction construction and signing

5. **Integration into Wallet**
   - Modified `send_to_address_internal` to detect and handle BitcoinZ transactions
   - Added debugging output for transaction analysis
   - Falls back gracefully when custom builders fail

### Successfully Resolved! ✅

**Solution**: Implemented custom BitcoinZ v4 transaction builder with correct sighash computation.

**Key Fixes**:
1. ✅ Use BitcoinZ's fixed consensus branch ID: `1991772603` (0x76b809bb)
2. ✅ Set expiry height to 0 (no expiry) 
3. ✅ Omit binding signature entirely for transparent-only transactions
4. ✅ **Critical**: Do NOT reverse the BLAKE2b sighash bytes after computation

**Successful Transaction Example**:
```
Transaction ID: 7ca9d30f60cf1e9d3be0cfefbaeccd713cf04b631722d5882e6ce9373cde6065
Type: Transparent-to-Transparent (t→t)
Amount: 0.1 BTCZ
Status: Successfully accepted by BitcoinZ network
```

### Implementation Details
The working implementation (`bitcoinz_v4_no_sig.rs`) creates v4 Sapling transactions that:
- Use correct BitcoinZ version group ID (0x892f2085)
- Compute sighash with BLAKE2b using BitcoinZ's personalization
- Handle transparent inputs and outputs correctly
- Are accepted by the BitcoinZ network

## Phase 2: Shielded Transactions (t→z, z→t, z→z)

### Status: Not Started

These will require fixing the binding signature issue first, as all Sapling transactions need proper binding signatures.

## Testing

### Test Wallet Info
- Address: `t1dWCXCaMn2tJqUuzxTPRNXfmaLQQVnYPcN`
- Balance: 1 BTCZ
- Server: `http://93.107.37.216:9067`

### Test Commands
```bash
# Build the project
cd /Users/mac/Code/mobile_wallet/src/lightwalletd/clean-repo/testclient/btcz-light-cli
cargo build

# Import wallet (need seed phrase)
cargo run -- -s http://93.107.37.216:9067 --data-dir /tmp/btcz-wallet import "seed phrase here"

# Check balance
cargo run -- -s http://93.107.37.216:9067 --data-dir /tmp/btcz-wallet balance

# Test t→t send
cargo run -- -s http://93.107.37.216:9067 --data-dir /tmp/btcz-wallet send t1JM4RcuaFKmYxiFj1Zptc3a96EQ5ktHiWD 0.1
```

## Code Structure

```
lib/src/
├── bitcoinz_transaction.rs      # Transaction type detection
├── bitcoinz_binding_sig.rs      # Binding signature analysis
├── bitcoinz_binding_sig_fix.rs  # Binding signature fix attempts
├── bitcoinz_branch.rs           # Branch ID for BitcoinZ
├── bitcoinz_overwinter.rs       # Overwinter format utilities
├── bitcoinz_overwinter_builder.rs # Overwinter transaction builder
└── lib.rs                       # BitcoinZMainNetwork implementation
```

## Summary

We've created a comprehensive framework for BitcoinZ support in the light client:
- Network parameters are correctly configured
- Transaction types are detected and analyzed
- The binding signature issue is well-understood
- Infrastructure is in place for custom transaction builders

The main blocker is the binding signature validation difference between BitcoinZ and Zcash. This requires either:
1. Implementing BitcoinZ's exact algorithm
2. Building transactions in a format that bypasses the issue (like Overwinter v3)
3. Collaboration with BitcoinZ developers to align implementations

Once t→t transactions work, the same fix can be applied to enable shielded transactions.
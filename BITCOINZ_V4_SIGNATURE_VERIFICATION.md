# BitcoinZ v4 Transaction Signature Verification

Based on analysis of the bitcore-lib-btcz library, here are the key findings about how BitcoinZ handles signature verification for v4 (Sapling) transactions:

## Key Differences from Standard Zcash Implementation

### 1. Consensus Branch ID
- BitcoinZ uses a custom consensus branch ID: `1991772603` (0x76B809BB in hex)
- This is hardcoded in `lib/transaction/sighash.js` line 161
- This differs from Zcash's standard Sapling branch IDs

### 2. Sighash Algorithm for v4 Transactions
- For transactions with version >= 4, BitcoinZ uses the Sapling sighash algorithm
- Implementation is in `sighashSapling` function (lines 22-169 in sighash.js)
- Uses BLAKE2b hashing with personalization strings
- Key personalization strings used:
  - 'ZcashPrevoutHash' for hashing previous outputs
  - 'ZcashSequencHash' for hashing sequence numbers
  - 'ZcashOutputsHash' for hashing outputs
  - 'ZcashSigHash' + consensus branch ID for final sighash

### 3. Transaction Structure
- v4 transactions support:
  - Transparent inputs/outputs (standard)
  - Sapling shielded spends/outputs (spendDescs, outputDescs)
  - Value balance for shielded pool
  - Binding signature (only required if shielded components exist)
  - Expiry height (nExpiryHeight)
  - Version group ID: `0x892f2085` (DEFAULT_VERSION_GROUP_ID)

### 4. Transparent-Only v4 Transactions
- For v4 transactions with ONLY transparent inputs/outputs:
  - No binding signature is required (line 424-425 in transaction.js)
  - Still uses Sapling sighash algorithm with BLAKE2b
  - Still includes nExpiryHeight and nVersionGroupId fields
  - valueBalance should be 0

### 5. Signature Verification Flow
1. Transaction version is checked (>= 4 triggers Sapling logic)
2. Sighash is computed using BLAKE2b with BitcoinZ's consensus branch ID
3. ECDSA signature verification is performed using the computed sighash
4. For transparent inputs, standard script verification applies

## Implementation Notes

The signature verification process:
```javascript
// In sighash.js
if (transaction.version >= 4) {
    return sighashSapling(transaction, sighashType, inputNumber, subscript);
}

// Sapling sighash uses:
// - BLAKE2b hashing
// - Personalization: 'ZcashSigHash' + consensusBranchId (1991772603)
// - Includes all Sapling-specific fields even for transparent-only txs
```

## Network Parameters
- BitcoinZ mainnet parameters (from networks.js):
  - pubkeyhash: 0x1cb8
  - scripthash: 0x1cbd
  - privkey: 0x80
  - Network magic: 0x24e92764
  - Port: 1989

## Verification Requirements
For a valid v4 transparent-only transaction:
1. Must have fOverwintered flag set
2. Must have correct nVersionGroupId (0x892f2085)
3. Must use Sapling sighash algorithm with BitcoinZ's branch ID
4. No binding signature required if no shielded components
5. Standard script verification for transparent inputs

This implementation follows the Zcash Sapling specification but with BitcoinZ-specific parameters, particularly the unique consensus branch ID.
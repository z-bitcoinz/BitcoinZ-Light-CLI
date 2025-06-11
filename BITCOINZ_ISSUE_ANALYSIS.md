# BitcoinZ Transaction Issue Analysis

## Problem Summary
BitcoinZ t→t transactions are failing with: `bad-txns-sapling-binding-signature-invalid`

## Root Cause Identified
1. **Zcash libraries compute a non-zero binding signature** even for transparent-only transactions
   - Example: `c5 de 8d 90 8f 98 b7 03 e5 9b 8f 0d 85 e0 c5 fb ...`
   
2. **BitcoinZ validates binding signatures differently**:
   - Zcash: `signature = sign(bsk, sighash)`
   - BitcoinZ: `signature = sign(bsk, bvk || sighash)`
   
3. **The binding signature cannot be simply zeroed out** - BitcoinZ actually validates it

## Current Status
- Your wallet has 1 BTCZ at address: `t1dWCXCaMn2tJqUuzxTPRNXfmaLQQVnYPcN`
- Attempting to send 0.1 BTCZ to: `t1JM4RcuaFKmYxiFj1Zptc3a96EQ5ktHiWD`
- Transaction creation succeeds but broadcasting fails

## Attempted Solutions
1. ✅ Force Overwinter height - Failed (causes "pre-overwinter signing not supported" error)
2. ✅ Zero out binding signature - Failed (BitcoinZ still validates it)
3. ✅ Analyze binding signature - Found it's non-zero and computed by Zcash libs

## Possible Solutions (Not Yet Implemented)
1. **Implement BitcoinZ's binding signature algorithm**
   - Requires modifying core crypto libraries
   - Need to compute: `sign(bsk, bvk || sighash)`
   
2. **Use legacy Bitcoin transaction format**
   - Completely bypass Sapling/Overwinter
   - Build version 1 transactions
   
3. **Fork and modify librustzcash**
   - Create BitcoinZ-specific version
   - Implement their binding signature algorithm

4. **Work with BitcoinZ developers**
   - Get their exact implementation
   - Or request they support standard Zcash format

## Technical Details
- Transaction version created: Sapling (0x80000004)
- Transaction size: ~1222 bytes
- Binding signature location: Last 64 bytes of transaction
- BitcoinZ server: http://93.107.37.216:9067

## Recommendation
The issue requires either:
1. Deep modification of Zcash cryptographic libraries
2. Implementation of a completely custom transaction builder
3. Collaboration with BitcoinZ developers to align implementations

This is a fundamental incompatibility between BitcoinZ and Zcash transaction formats.
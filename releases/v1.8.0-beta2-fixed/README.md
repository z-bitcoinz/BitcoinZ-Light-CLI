# BitcoinZ Light CLI v1.8.0-beta2-fixed

## 🎉 SHIELDED TRANSACTIONS WORKING

This release fixes the critical issue where shielded transactions were failing with:
```
"16: bad-txns-sapling-output-description-invalid"
```

## What's Fixed

✅ **All transaction types now working**:
- Transparent → Transparent (t→t)
- **Transparent → Shielded (t→z) - FIXED!**
- Shielded → Shielded (z→z)  
- Shielded → Transparent (z→t)

## Verified Working Transaction

**Transaction ID**: `f2a573939911115cb4f33c0fd54014626df87c63c278c7fee11dffa786ce8a99`
- Successfully sent to shielded address
- Network accepted without errors
- Full privacy protection enabled

## Download

**macOS ARM64 (Apple Silicon)**:
- Binary: `bitcoinz-light-cli-v1.8.0-beta2-fixed-macos-arm64.tar.gz`
- SHA256: See `.sha256` file for verification

## Usage

```bash
# Extract
tar -xzf bitcoinz-light-cli-v1.8.0-beta2-fixed-macos-arm64.tar.gz

# Make executable
chmod +x bitcoinz-light-cli

# Test shielded transaction (this now works!)
./bitcoinz-light-cli --server https://lightd.btcz.rocks:9067 send zs1... 1000
```

## Technical Details

- **Consensus Branch ID**: Fixed to use correct `0x76b809bb` 
- **Builder**: Uses standard zcash_primitives Builder (same as BitcoinZ Blue)
- **Network**: BitcoinZ Mainnet compatible
- **Server**: Compatible with `https://lightd.btcz.rocks:9067`

## Build Info

- **Built from**: Latest source code with correct consensus parameters
- **Date**: August 7, 2025
- **Platform**: macOS ARM64 (Apple Silicon)
- **Rust version**: 1.88.0

**Ready for production use!** 🚀
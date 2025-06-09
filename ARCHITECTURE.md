# BitcoinZ Light Wallet - Architecture & Naming

## Why "BitcoinZ Light CLI"?

This wallet uses the **lightwalletd protocol**, which is a "light" client architecture:

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  BitcoinZ Light │────▶│   lightwalletd   │────▶│  BitcoinZ Node  │
│       CLI       │     │    (Bridge)      │     │   (Full Node)   │
└─────────────────┘     └──────────────────┘     └─────────────────┘
     Your Wallet         Port 9067              Port 1979
```

### Light Client vs Full Node Wallet

**Light Client (This Wallet):**
- ✅ Fast sync (minutes instead of days)
- ✅ Small storage requirement (~100MB)
- ✅ Privacy preserved (shielded addresses)
- ✅ Connects via lightwalletd bridge
- ⚠️  Requires trust in lightwalletd server

**Full Node Wallet:**
- ❌ Slow sync (downloads entire blockchain ~40GB)
- ❌ Large storage requirement
- ✅ Maximum security (no trust required)
- ✅ Direct blockchain connection

### Naming Convention

- **"Light"** = Uses lightwalletd protocol (not full blockchain)
- **"CLI"** = Command Line Interface
- **"BitcoinZ"** = The cryptocurrency network

### Similar Projects
- Zecwallet Light (Zcash)
- Nighthawk Wallet (Zcash) 
- YWallet (Ycash/Zcash)

All use the same lightwalletd protocol architecture.

### GitHub Repository Names

**Good names for light wallets:**
- `bitcoinz-light-cli` ✅ (Recommended)
- `bitcoinz-light-wallet` ✅
- `btcz-light` ✅

**Avoid these names:**
- `bitcoinz-wallet` (ambiguous - could be full node)
- `bitcoinz-core` (implies full node)
- `bitcoinz-qt` (implies GUI wallet)

### For Production Use

Users will need:
1. A lightwalletd server to connect to
2. The wallet binary (this project)

Example servers:
- Local: `http://localhost:9067`
- Production: `https://lightwalletd.bitcoinz.org:9067` (future)

This architecture allows fast, efficient wallets while maintaining privacy!

#!/bin/bash

echo "Testing BitcoinZ t→t transaction with Overwinter fix..."
echo ""
echo "Sending 0.1 BTCZ from t1dWCXCaMn2tJqUuzxTPRNXfmaLQQVnYPcN to t1JM4RcuaFKmYxiFj1Zptc3a96EQ5ktHiWD"
echo ""

# Send the transaction
echo "send t1JM4RcuaFKmYxiFj1Zptc3a96EQ5ktHiWD 10000000" | ./target/release/bitcoinz-light-cli --server=http://93.107.37.216:9067
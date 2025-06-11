#!/bin/bash

echo "Checking BitcoinZ wallet balance..."
echo "balance" | ./target/release/bitcoinz-light-cli --server=http://93.107.37.216:9067 | grep -A 20 "balance"
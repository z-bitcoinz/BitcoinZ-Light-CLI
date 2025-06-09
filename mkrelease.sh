#!/bin/bash
# This script depends on a docker image already being built
# To build it, 
# cd docker
# docker build --tag rustbuild:latest .

POSITIONAL=()
while [[ $# -gt 0 ]]
do
key="$1"

case $key in
    -v|--version)
    APP_VERSION="$2"
    shift # past argument
    shift # past value
    ;;
    *)    # unknown option
    POSITIONAL+=("$1") # save it in an array for later
    shift # past argument
    ;;
esac
done
set -- "${POSITIONAL[@]}" # restore positional parameters

if [ -z $APP_VERSION ]; then echo "APP_VERSION is not set"; exit 1; fi

# Write the version file
echo "pub const VERSION: &str = \"$APP_VERSION\";" > cli/src/version.rs

# First, do the tests
cd lib && cargo test --release
retVal=$?
if [ $retVal -ne 0 ]; then
    echo "Error"
    exit $retVal
fi
cd ..

# Compile for mac directly
cargo build --release 

#macOS
codesign -f -s "Apple Distribution: Concision Systems LLC (5N76B7JDDT)" target/release/bitcoinz-wallet-cli --deep
rm -rf target/macOS-bitcoinz-wallet-cli-v$APP_VERSION
mkdir -p target/macOS-bitcoinz-wallet-cli-v$APP_VERSION
cp target/release/bitcoinz-wallet-cli target/macOS-bitcoinz-wallet-cli-v$APP_VERSION/

# Now sign and zip the binaries
# macOS
gpg --batch --output target/macOS-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli.sig --detach-sig target/macOS-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli 
cd target
cd macOS-bitcoinz-wallet-cli-v$APP_VERSION
gsha256sum bitcoinz-wallet-cli > sha256sum.txt
cd ..
zip -r macOS-bitcoinz-wallet-cli-v$APP_VERSION.zip macOS-bitcoinz-wallet-cli-v$APP_VERSION 
cd ..

# For Windows and Linux, build via docker
docker run --rm -v $(pwd)/:/opt/zecwallet-light-cli rustbuild:latest bash -c "cd /opt/zecwallet-light-cli && cargo build --release && SODIUM_LIB_DIR='/opt/libsodium-win64/lib/' cargo build --release --target x86_64-pc-windows-gnu"

#Linux
rm -rf target/linux-bitcoinz-wallet-cli-v$APP_VERSION
mkdir -p target/linux-bitcoinz-wallet-cli-v$APP_VERSION
cp target/release/bitcoinz-wallet-cli target/linux-bitcoinz-wallet-cli-v$APP_VERSION/
gpg --batch --output target/linux-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli.sig --detach-sig target/linux-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli
cd target
cd linux-bitcoinz-wallet-cli-v$APP_VERSION
gsha256sum bitcoinz-wallet-cli > sha256sum.txt
cd ..
zip -r linux-bitcoinz-wallet-cli-v$APP_VERSION.zip linux-bitcoinz-wallet-cli-v$APP_VERSION 
cd ..


#Windows
rm -rf target/Windows-bitcoinz-wallet-cli-v$APP_VERSION
mkdir -p target/Windows-bitcoinz-wallet-cli-v$APP_VERSION
cp target/x86_64-pc-windows-gnu/release/bitcoinz-wallet-cli.exe target/Windows-bitcoinz-wallet-cli-v$APP_VERSION/
gpg --batch --output target/Windows-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli.sig --detach-sig target/Windows-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli.exe
cd target
cd Windows-bitcoinz-wallet-cli-v$APP_VERSION
gsha256sum bitcoinz-wallet-cli.exe > sha256sum.txt
cd ..
zip -r Windows-bitcoinz-wallet-cli-v$APP_VERSION.zip Windows-bitcoinz-wallet-cli-v$APP_VERSION 
cd ..


# #Armv7
# rm -rf target/Armv7-bitcoinz-wallet-cli-v$APP_VERSION
# mkdir -p target/Armv7-bitcoinz-wallet-cli-v$APP_VERSION
# cp target/armv7-unknown-linux-gnueabihf/release/bitcoinz-wallet-cli target/Armv7-bitcoinz-wallet-cli-v$APP_VERSION/
# gpg --batch --output target/Armv7-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli.sig --detach-sig target/Armv7-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli
# cd target
# cd Armv7-bitcoinz-wallet-cli-v$APP_VERSION
# gsha256sum bitcoinz-wallet-cli > sha256sum.txt
# cd ..
# zip -r Armv7-bitcoinz-wallet-cli-v$APP_VERSION.zip Armv7-bitcoinz-wallet-cli-v$APP_VERSION 
# cd ..


# #AARCH64
# rm -rf target/aarch64-bitcoinz-wallet-cli-v$APP_VERSION
# mkdir -p target/aarch64-bitcoinz-wallet-cli-v$APP_VERSION
# cp target/aarch64-unknown-linux-gnu/release/bitcoinz-wallet-cli target/aarch64-bitcoinz-wallet-cli-v$APP_VERSION/
# gpg --batch --output target/aarch64-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli.sig --detach-sig target/aarch64-bitcoinz-wallet-cli-v$APP_VERSION/bitcoinz-wallet-cli
# cd target
# cd aarch64-bitcoinz-wallet-cli-v$APP_VERSION
# gsha256sum bitcoinz-wallet-cli > sha256sum.txt
# cd ..
# zip -r aarch64-bitcoinz-wallet-cli-v$APP_VERSION.zip aarch64-bitcoinz-wallet-cli-v$APP_VERSION 
# cd ..

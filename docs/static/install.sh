#!/usr/bin/env sh
set -euo pipefail

fail() {
    echo $1
    exit 1
}

# CONFIGURE VARIABLES
REPO="https://github.com/daniel7grant/gw"
VERSION="v0.3.0"
if [ "$(id -u)" -ne "0" ]; then
    BIN_DIR="$HOME/.local/bin"
else
    BIN_DIR="/usr/local/bin"
fi
if ldd /bin/ls | grep -q "musl"; then
    LIBC="musl"
else
    LIBC="gnu"
fi

# DETERMINE THE CORRECT FILENAME
PLATFORM=$(uname -sm)
case "$PLATFORM" in
    "Linux x86_64")
        FILE="gw-bin_x86_64-unknown-linux-$LIBC.zip"
        ;;
    "Linux aarch"* | "Linux arm"*)
        FILE="gw-bin_arm-unknown-linux-gnueabihf.zip"
        ;;
    *)
        fail "Platform $PLATFORM is currently not supported."
        ;;
esac

# DOWNLOAD AND MOVE IT TO BIN_DIR
echo "Downloading version $VERSION to $PLATFORM..."
DOWNLOAD_URL="$REPO/releases/download/$VERSION/$FILE"
curl -Lfq --progress-bar $DOWNLOAD_URL -o $FILE || fail "Failed to download $DOWNLOAD_URL."
unzip -qo $FILE || fail "Failed to unzip $FILE."
mv gw "$BIN_DIR/gw"
rm $FILE

echo "Successfully installed gw binary to $BIN_DIR/gw!"

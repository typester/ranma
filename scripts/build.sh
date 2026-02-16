#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

CONFIGURATION="${1:-debug}"

case "$CONFIGURATION" in
    release)
        CARGO_FLAGS="--release"
        CARGO_OUT="target/release"
        SWIFT_FLAGS="-c release"
        ;;
    *)
        CARGO_FLAGS=""
        CARGO_OUT="target/debug"
        SWIFT_FLAGS="-c debug"
        ;;
esac

echo "==> Building ranma-core (Rust, $CONFIGURATION)..."
cargo build $CARGO_FLAGS -p ranma-core

echo "==> Building ranma CLI (Rust, $CONFIGURATION)..."
cargo build $CARGO_FLAGS -p ranma-cli

echo "==> Generating UniFFI Swift bindings..."
cargo run -p uniffi-bindgen -- generate \
    --library "$CARGO_OUT/libranma_core.a" \
    --language swift \
    --out-dir app/Sources/Generated/

echo "==> Copying C headers for Swift systemLibrary target..."
cp app/Sources/Generated/ranma_coreFFI.h app/Sources/CRanmaCore/include/

# SPM requires module.modulemap at the system library root.
# Module name must be "ranma_coreFFI" to match the generated Swift import.
cat > app/Sources/CRanmaCore/module.modulemap <<'MODULEMAP'
module ranma_coreFFI {
    header "include/ranma_coreFFI.h"
    export *
}
MODULEMAP

echo "==> Building Swift app ($CONFIGURATION)..."
cd app
swift build $SWIFT_FLAGS \
    -Xlinker -L"$ROOT_DIR/$CARGO_OUT" \
    -Xlinker -lranma_core

echo "==> Done."
echo "  Rust CLI: $ROOT_DIR/$CARGO_OUT/ranma"
echo "  Swift app: app/.build/$CONFIGURATION/Ranma"

#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

TARGET=""
CONFIGURATION="debug"

usage() {
    echo "Usage: $0 [--target <triple>] [debug|release]"
    echo ""
    echo "Options:"
    echo "  --target <triple>  Cross-compile target (e.g., x86_64-apple-darwin)"
    echo "  debug              Debug build (default)"
    echo "  release            Release build"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            TARGET="$2"
            shift 2
            ;;
        debug|release)
            CONFIGURATION="$1"
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

CARGO_ARGS=()
SWIFT_FLAGS=()

if [[ -n "$TARGET" ]]; then
    CARGO_ARGS+=(--target "$TARGET")
    CARGO_OUT="target/${TARGET}/${CONFIGURATION}"
else
    CARGO_OUT="target/${CONFIGURATION}"
fi

case "$CONFIGURATION" in
    release)
        CARGO_ARGS+=(--release)
        SWIFT_FLAGS+=(-c release)
        ;;
    *)
        SWIFT_FLAGS+=(-c debug)
        ;;
esac

echo "==> Building ranma-core (Rust, $CONFIGURATION)..."
cargo build ${CARGO_ARGS[@]+"${CARGO_ARGS[@]}"} -p ranma-core

echo "==> Building ranma CLI (Rust, $CONFIGURATION)..."
cargo build ${CARGO_ARGS[@]+"${CARGO_ARGS[@]}"} -p ranma-cli

echo "==> Generating UniFFI Swift bindings..."
cargo run -p uniffi-bindgen -- generate \
    --library "$CARGO_OUT/libranma_core.a" \
    --language swift \
    --out-dir app/Sources/Generated/

echo "==> Copying C headers for Swift systemLibrary target..."
cp app/Sources/Generated/ranma_coreFFI.h app/Sources/CRanmaCore/include/

cat > app/Sources/CRanmaCore/module.modulemap <<'MODULEMAP'
module ranma_coreFFI {
    header "include/ranma_coreFFI.h"
    export *
}
MODULEMAP

echo "==> Building Swift app ($CONFIGURATION)..."
cd app

SWIFT_LINK_FLAGS=(
    -Xlinker -L"$ROOT_DIR/$CARGO_OUT"
    -Xlinker -lranma_core
)

if [[ -n "$TARGET" ]]; then
    case "$TARGET" in
        aarch64-apple-darwin)
            SWIFT_FLAGS+=(--arch arm64)
            ;;
        x86_64-apple-darwin)
            SWIFT_FLAGS+=(--arch x86_64)
            ;;
    esac
fi

swift build ${SWIFT_FLAGS[@]+"${SWIFT_FLAGS[@]}"} "${SWIFT_LINK_FLAGS[@]}"

cp "$ROOT_DIR/app/.build/$CONFIGURATION/ranma-server" "$ROOT_DIR/$CARGO_OUT/"

echo "==> Done."
echo "  ranma:        $ROOT_DIR/$CARGO_OUT/ranma"
echo "  ranma-server: $ROOT_DIR/$CARGO_OUT/ranma-server"

#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LLVM_TESTS_DIR="$PROJECT_ROOT/llvm-tests"

echo "Downloading LLVM test suite..."

# Create directory if it doesn't exist
mkdir -p "$LLVM_TESTS_DIR"
cd "$LLVM_TESTS_DIR"

# Clone LLVM project if not already present
if [ ! -d "llvm-project" ]; then
    echo "Cloning LLVM project (this will take a few minutes)..."
    git clone --depth 1 --filter=blob:none --sparse https://github.com/llvm/llvm-project.git
    cd llvm-project
    git sparse-checkout set llvm/test
else
    echo "LLVM project already cloned, updating..."
    cd llvm-project
    git pull
fi

# Create symlink for easy access
cd "$LLVM_TESTS_DIR"
if [ ! -L "llvm" ]; then
    ln -s llvm-project/llvm llvm
fi

echo "LLVM test suite downloaded successfully!"
echo "Test directory: $LLVM_TESTS_DIR/llvm/test"
echo ""
echo "Available test directories:"
ls -la "$LLVM_TESTS_DIR/llvm/test" | head -20

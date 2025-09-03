#!/bin/bash

set -e

echo "Generating code coverage report..."
cargo llvm-cov --all-features --workspace --html --open

echo "Coverage report generated and opened in your browser"

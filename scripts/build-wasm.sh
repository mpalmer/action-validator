#!/usr/bin/env bash

set -euo pipefail

npx wasm-pack build --out-dir target/wasm-pack/build --no-typescript --target nodejs --features js
rm -rf packages/core/snippets
cp -R target/wasm-pack/build/snippets packages/core/snippets
cp target/wasm-pack/build/action_validator_bg.wasm packages/core/
cp target/wasm-pack/build/action_validator.js packages/core/

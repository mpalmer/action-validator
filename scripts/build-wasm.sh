#!/usr/bin/env bash

set -euo pipefail

# [SC2086] Intenitonally omitting double quotes to allow word splitting of WASM_PACK_BUILD_FLAGS
# shellcheck disable=SC2086
npx wasm-pack build ${WASM_PACK_BUILD_FLAGS:-} --out-dir target/wasm-pack/build --no-typescript --target nodejs --no-default-features --features js
rm -rf packages/core/snippets
cp -R target/wasm-pack/build/snippets packages/core/snippets
cp target/wasm-pack/build/action_validator_bg.wasm packages/core/
cp target/wasm-pack/build/action_validator.js packages/core/

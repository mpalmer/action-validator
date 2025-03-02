#!/usr/bin/env bash

set -euo pipefail

npm run build
cargo test -F test-js

#!/usr/bin/env bash

set -euo pipefail

cargo test -F test-js -- --nocapture

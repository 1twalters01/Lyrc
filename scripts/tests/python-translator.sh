#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR"

while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do
    ROOT="$(dirname "$ROOT")"
done

if [[ ! -f "$ROOT/Cargo.toml" ]]; then
    echo "Could not find workspace root" >&2
    exit 1
fi

cd "$ROOT/python/translator"

uv run --env-file .env pytest -s


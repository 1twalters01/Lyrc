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

export VIRTUAL_ENV="$ROOT/python/.venv"
export PYTHONHOME="$($VIRTUAL_ENV/bin/python -c 'import sys; print(sys.base_prefix)')"

export PYTHONPATH="$(uv --directory "$ROOT/python" run python -c '
import sys
print(":".join(
    p for p in sys.path
    if "site-packages" in p or "/src" in p
))
')"

cd "$ROOT"

cargo test -p translation

#!/usr/bin/env bash
# Build locally and deploy an atomic static release to the SSH host in ~/.ssh/config.
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
HOST=${SUDOKU_DEPLOY_HOST:-v}
TRUNK=${TRUNK:-trunk}
RELEASE="$(date -u +%Y%m%dT%H%M%SZ)-$$"
ACTIVATING=0

cleanup() {
    if [[ $ACTIVATING == 0 ]]; then
        ssh -o BatchMode=yes "$HOST" 'rm -rf -- "$HOME/sudoku402/releases/'"$RELEASE"'"' >/dev/null 2>&1 || true
    fi
}
trap cleanup EXIT

cd "$ROOT"
"$TRUNK" build --release
test -f dist/index.html
test -f dist/help/index.html

ssh -o BatchMode=yes "$HOST" 'umask 022; mkdir -p -- "$HOME/sudoku402/releases/'"$RELEASE"'"'
tar -C dist -czf - . | ssh -o BatchMode=yes "$HOST" 'umask 022; tar -xzf - -C "$HOME/sudoku402/releases/'"$RELEASE"'"'
# If the connection drops after the atomic switch, its outcome is unknown;
# preserve the release rather than risk deleting the active directory.
ACTIVATING=1
ssh -o BatchMode=yes "$HOST" bash -s -- "$RELEASE" <<'REMOTE'
set -euo pipefail
release=$1
root="$HOME/sudoku402"
test -f "$root/releases/$release/index.html"
test -f "$root/releases/$release/help/index.html"
# The web server follows current; do not replace a pre-existing real directory.
if [[ -e "$root/current" && ! -L "$root/current" ]]; then
    echo "Refusing to replace non-symlink $root/current" >&2
    exit 1
fi
ln -s "releases/$release" "$root/current.$release"
mv -Tf -- "$root/current.$release" "$root/current"
printf 'Active release: %s\n' "$root/releases/$release"
REMOTE
printf 'Deployed to %s:~/sudoku402/current\n' "$HOST"

#!/usr/bin/env bash
#
# Install/update the buzz-acp harness on the Cofoundy agent box.
#
# Pulls the newest binary published by .github/workflows/buzz-acp-linux.yml
# rather than building locally, so the host needs no Rust toolchain and the
# running binary is always traceable to a commit.
#
# Usage:
#   ./install-buzz-acp.sh              # newest published build
#   ./install-buzz-acp.sh <short-sha>  # pin a specific build
#
set -euo pipefail

REPO="${BUZZ_ACP_REPO:-cofoundy/buzz}"
DEST="${BUZZ_ACP_DEST:-$HOME/.local/bin}"
# Both binaries: the harness's base prompt makes the `buzz` CLI the agent's
# primary interface, so installing buzz-acp alone yields an agent that runs
# turns and silently never replies.
ASSET="buzz-acp-x86_64-linux"
CLI_ASSET="buzz-x86_64-linux"

pinned="${1:-}"
if [ -n "$pinned" ]; then
    tag="buzz-acp-linux-${pinned#buzz-acp-linux-}"
else
    # Releases are listed newest-first; filter to this workflow's tag namespace
    # so unrelated upstream releases (desktop builds, etc.) never match.
    #
    # Read tagName via --json, never the human table: its first column is the
    # release *title*, so column-parsing silently yields the wrong string.
    tag="$(gh release list --repo "$REPO" --limit 50 --json tagName -q '.[].tagName' \
        | grep '^buzz-acp-linux-' | head -1)"
    [ -n "$tag" ] || { echo "no buzz-acp release found in $REPO — run the workflow first" >&2; exit 1; }
fi

echo "installing $tag from $REPO"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

gh release download "$tag" --repo "$REPO" --pattern "*x86_64-linux*" --dir "$tmp" --clobber

# Verify before installing: a truncated download would otherwise land as a
# broken binary that only fails at agent start time.
( cd "$tmp" && sha256sum -c "${ASSET}.sha256" )

mkdir -p "$DEST"
install -m 0755 "$tmp/$ASSET" "$DEST/buzz-acp"
install -m 0755 "$tmp/$CLI_ASSET" "$DEST/buzz"

echo "installed: $DEST/buzz-acp + $DEST/buzz ($tag)"
"$DEST/buzz-acp" --help | head -1
"$DEST/buzz" --help | head -1

cat <<EOF

Next:
  systemctl --user restart buzz-acp   # if the unit is already installed
  systemctl --user status buzz-acp
EOF

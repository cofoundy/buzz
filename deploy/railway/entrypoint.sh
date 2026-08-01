#!/bin/bash
#
# Fix volume ownership, then drop privileges and hand off to the relay.
#
# Railway (like a fresh Docker Compose named volume) mounts the persistent
# volume root-owned, but the upstream image runs buzz-relay as `buzz`
# (uid 1000). The relay cannot then create BUZZ_GIT_PACK_CACHE_PATH inside it
# and crash-loops on:
#
#   invalid config: BUZZ_GIT_PACK_CACHE_PATH=/data/git/.pack-cache
#   could not be created: Permission denied (os error 13)
#
# Upstream issue: https://github.com/block/buzz/issues/2814
#
set -euo pipefail

GIT_PATH="${BUZZ_GIT_REPO_PATH:-/data/git}"

# Only chown when we actually booted as root — if the platform already starts us
# as buzz, the volume was never ours to fix and re-exec would be a no-op anyway.
if [ "$(id -u)" = "0" ]; then
    mkdir -p "$GIT_PATH"
    chown -R buzz:buzz "$GIT_PATH"
    exec gosu buzz:buzz /usr/local/bin/buzz-relay "$@"
fi

exec /usr/local/bin/buzz-relay "$@"

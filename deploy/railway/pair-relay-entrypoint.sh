#!/bin/bash
#
# Bind the NIP-AB pairing sidecar to the port Railway actually routes to.
#
# buzz-pair-relay defaults to 127.0.0.1:5000 (crates/buzz-pair-relay/src/main.rs).
# Both halves of that default are wrong on Railway:
#
#   - 127.0.0.1 accepts only loopback, so the edge proxy cannot reach it and
#     every request 502s while the container looks perfectly healthy.
#   - the port is assigned at runtime via $PORT, so it cannot be baked into an
#     ENV line at build time.
#
# Hence a shell entrypoint rather than a Dockerfile ENV.
set -euo pipefail

PORT="${PORT:-5000}"

# Respect an explicit override, but default to all interfaces on Railway's port.
export BUZZ_PAIR_RELAY_BIND_ADDR="${BUZZ_PAIR_RELAY_BIND_ADDR:-0.0.0.0:${PORT}}"

echo "buzz-pair-relay: binding ${BUZZ_PAIR_RELAY_BIND_ADDR}"

# exec so the relay is PID 1's direct target and receives SIGTERM on shutdown.
exec /usr/local/bin/buzz-pair-relay "$@"

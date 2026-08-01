# Thin deploy wrapper over the published Buzz relay image (Cofoundy / Railway).
#
# This does NOT rebuild the relay — it inherits the prebuilt upstream image, so
# it builds in seconds instead of compiling the Rust workspace.
#
# The only thing it adds is a root-side entrypoint that fixes ownership of the
# mounted git volume before dropping back to the non-root `buzz` user. Without
# it the relay crash-loops on a root-owned volume (block/buzz#2814); see
# deploy/railway/entrypoint.sh for the full rationale.
#
# Bumping the relay: change BUZZ_IMAGE to a newer immutable `sha-<short>` tag
# (find one under the repo's "Docker image" workflow runs). Never point this at
# `:latest` or `:main` — a redeploy would then silently jump versions and the
# running code would stop matching what any doc claims.
ARG BUZZ_IMAGE=ghcr.io/block/buzz:sha-b1b283c
FROM ${BUZZ_IMAGE}

USER root

# gosu drops privileges without the signal/TTY weirdness of `su`, so the relay
# still runs as PID 1's direct exec target and receives SIGTERM on shutdown.
RUN apt-get update \
    && apt-get install -y --no-install-recommends gosu \
    && rm -rf /var/lib/apt/lists/*

COPY deploy/railway/entrypoint.sh /usr/local/bin/buzz-railway-entrypoint.sh
RUN chmod 0755 /usr/local/bin/buzz-railway-entrypoint.sh

# Inherited from the base image, restated so this file is self-describing:
# 3000 = app (WS + REST) · 8080 = /_liveness, /_readiness · 9102 = /metrics
EXPOSE 3000 8080 9102

ENTRYPOINT ["/usr/local/bin/buzz-railway-entrypoint.sh"]

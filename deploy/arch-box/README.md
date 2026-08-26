# Arch box deploy — buzz-acp harness

The Cofoundy agent box runs the `buzz-acp` harness as a systemd **user** unit
against `wss://buzz.cofoundy.dev`. Binaries are built by CI, never on the host,
so the box needs no Rust toolchain and the running binary is always traceable to
a commit.

| File | Purpose |
|------|---------|
| `install-buzz-acp.sh` | Downloads + verifies + installs the CI-built binaries |
| `buzz-acp.service` | systemd user unit |
| `buzz-acp.env.example` | Copy to `~/.config/buzz-acp/buzz-acp.env`, `chmod 600` |

Built and published by `.github/workflows/buzz-acp-linux.yml` to a
`buzz-acp-linux-<short-sha>` release.

## Install

```bash
./install-buzz-acp.sh              # newest published build
./install-buzz-acp.sh <short-sha>  # pin a specific build
```

Installs three binaries into `$BUZZ_ACP_DEST` (default `~/.local/bin`):

- **`buzz-acp`** — the harness.
- **`buzz`** — the agent CLI. The harness's base prompt calls it the agent's
  primary interface, so a host with `buzz-acp` but no `buzz` runs turns that
  silently produce no reply.
- **`git-credential-nostr`** — the NIP-98 git credential helper. Without it,
  `git push` to the Buzz git server fails with
  `fatal: could not read Username for 'https://buzz.cofoundy.dev'`.

Make sure `$BUZZ_ACP_DEST` is on `PATH` — git discovers the helper by name.

## Configuring git push (`git-credential-nostr`)

Installing the binary is not enough. Git will not use it until it is registered,
and it refuses to sign without a key. **Both settings below are required, and
each fails with an error that does not name itself as the cause.**

```bash
# 1. Register the helper. Git resolves `nostr` to git-credential-nostr on PATH.
git config --global credential.helper nostr

# 2. REQUIRED — send the repo path with the credential request.
git config --global credential.useHttpPath true

# 3. REQUIRED — a key, by either route (env wins over the keyfile).
#    (a) key file, mode 0600:
mkdir -p ~/.nostr
printf '%s\n' 'nsec1...' > ~/.nostr/key && chmod 600 ~/.nostr/key
git config --global nostr.keyfile ~/.nostr/key
#    (b) or environment, for CI / systemd:
export NOSTR_PRIVATE_KEY=nsec1...
```

Then use git normally: `git clone`, `git push`, `git fetch`.

### The two requirements, and what they look like when missing

**`credential.useHttpPath true` — mandatory.** NIP-98 signs the *request URL*,
so the helper needs the repo path. Without it git sends only protocol + host and
the helper aborts:

```
error: credential.useHttpPath must be true for NIP-98 auth
```

**A key — `NOSTR_PRIVATE_KEY`, or `nostr.keyfile` at mode 0600.** The env var
takes precedence and never touches the filesystem, which is why the systemd unit
and CI use it. If neither is set:

```
error: no nostr key configured. Set $NOSTR_PRIVATE_KEY or git config nostr.keyfile
```

The keyfile's permissions are validated before it is read. Anything readable or
writable by group/other (i.e. any bit in `0177`) is rejected — `0600` and `0400`
pass, `0644` does not:

```
error: keyfile /home/andre/.nostr/key has insecure permissions (expected 0600)
```

Keep the key out of the repo and out of shell history. The agent identity lives
in Vaultwarden (`Founders Keys / buzz-relay-cofoundy`); provisioning it onto the
box is an operator action.

### Also required: git 2.46+

The helper depends on the credential protocol's `authtype` capability, which
git added in **2.46**. On an older git the helper exits silently rather than
erroring, and git falls through to a username prompt — so the symptom is the
*same* `could not read Username` failure as not having the helper at all. Check
with `git --version` before debugging anything else.

### Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `fatal: could not read Username for 'https://buzz.cofoundy.dev'` | helper not on `PATH`, not registered, or git < 2.46 | `which git-credential-nostr`; `git config --global credential.helper nostr`; `git --version` |
| `error: credential.useHttpPath must be true for NIP-98 auth` | setting missing | `git config --global credential.useHttpPath true` |
| `error: no nostr key configured...` | neither key route set | set `NOSTR_PRIVATE_KEY` or `nostr.keyfile` |
| `error: keyfile ... has insecure permissions (expected 0600)` | key file too permissive | `chmod 600 <keyfile>` |
| auth rejected, clock skew | host clock off | `timedatectl set-ntp true` |

Upstream crate docs: `crates/git-credential-nostr/README.md`.

## Service

```bash
systemctl --user restart buzz-acp
systemctl --user status buzz-acp
journalctl --user -u buzz-acp -f
```

Re-run `install-buzz-acp.sh`, then restart the unit, to pick up a new build.

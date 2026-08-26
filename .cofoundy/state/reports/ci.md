# Lane `ci` — T-001 report

**Branch:** `lane/ci-git-credential-nostr` (off `sprint/buzz-cli-usable`)
**Commit:** `f40de5a16` — DCO signed off
**Status:** all four acceptance lines pass, evidence below.

## What changed

| File | Change |
|---|---|
| `.github/workflows/buzz-acp-linux.yml` | `paths:` + `crates/git-credential-nostr/**`; `cargo build` + `-p git-credential-nostr`; smoke check + helper; asset `git-credential-nostr-x86_64-linux` into the existing `sha256sum` and both `gh release` branches |
| `deploy/arch-box/install-buzz-acp.sh` | `HELPER_ASSET` + `install -m 0755` → `$DEST/git-credential-nostr`, added to the final echo and the verification run |
| `deploy/arch-box/README.md` | **new** — deploy guide + the two config requirements |

Tag scheme `buzz-acp-linux-<sha>` unchanged, as required.
`crates/git-credential-nostr/` read only, never modified.

## Acceptance

### 1. Workflow builds, smoke-checks and publishes all three; `paths:` includes the crate

Green run on this branch: [`30936986998`](https://github.com/cofoundy/buzz/actions/runs/30936986998)
— every step ✓, `gh run watch --exit-status` returned 0.

From the run log:

```
Run cargo build --release -p buzz-acp -p buzz-cli -p git-credential-nostr
   Compiling git-credential-nostr v0.1.0 (/home/runner/work/buzz/buzz/crates/git-credential-nostr)
Smoke check:
  ./target/release/buzz-acp --help > /dev/null
  ./target/release/buzz --help > /dev/null
  ./target/release/git-credential-nostr --help > /dev/null
```

`paths:` parsed back out of the YAML (no `pyyaml` in this env — used ruby):
`["crates/buzz-acp/**", "crates/buzz-sdk/**", "crates/buzz-ws-client/**",`
`"crates/git-credential-nostr/**", "rust-toolchain.toml", ".github/workflows/buzz-acp-linux.yml"]`
→ R-2 closed: a change to the helper now redispatches the build.

### 2. Installer installs the three into `$DEST`

`bash -n` clean, `shellcheck` clean. The asset name in the script
(`git-credential-nostr-x86_64-linux`) is byte-identical to what the workflow
published, and the existing download pattern `*x86_64-linux*` matches it —
proven by actually downloading the release with that pattern (all four files
came down). The pre-existing `sha256sum -c` now covers the helper for free:

```
$ shasum -a 256 -c buzz-acp-x86_64-linux.sha256
buzz-acp-x86_64-linux: OK
buzz-x86_64-linux: OK
git-credential-nostr-x86_64-linux: OK          exit=0
```

Downloaded helper is a real target binary, not a truncated artifact:
`ELF 64-bit LSB pie executable, x86-64 … for GNU/Linux 3.2.0`.

**Not executed end-to-end**: this lane runs on macOS, so the script's actual
`install` of Linux ELFs onto a host was not run. That is the arch box, and it is
T-003's gate.

### 3. `deploy/arch-box/README.md` documents both requirements, verified against the crate

Not copied by eye — every documented error string was **reproduced against a
locally built binary** (`cargo build --release -p git-credential-nostr`) by
feeding it the git credential protocol on stdin:

| Documented cause | Reproduced output |
|---|---|
| `credential.useHttpPath` unset | `error: credential.useHttpPath must be true for NIP-98 auth` |
| keyfile at `0644` | `error: keyfile <path> has insecure permissions (expected 0600)` |
| no key at all | `error: no nostr key configured. Set $NOSTR_PRIVATE_KEY or git config nostr.keyfile` |

Permission rule stated precisely from `lib.rs:33` (`mode & 0o177 != 0`), so the
README says `0600` **and** `0400` pass — not the vaguer "must be 0600".

Two additions beyond the two required requirements, both because they produce
the *same* symptom as the bug T-001 exists to fix and would otherwise send the
T-003 tester down the wrong path:

- `git config credential.helper nostr` — without registering it, the helper is
  installed but never invoked.
- **git 2.46+** — the helper needs the credential protocol's `authtype`
  capability (`lib.rs:118`). On older git it exits *silently* (`lib.rs:160-163`)
  and git falls through to a username prompt, i.e. the identical
  `could not read Username` failure as having no helper at all.

### 4. Green run + release carries the new asset with its sha256

Verified with `gh release view`, not from the job log:

```
$ gh release view buzz-acp-linux-f40de5a --repo cofoundy/buzz --json tagName,assets
buzz-acp-linux-f40de5a
buzz-acp-x86_64-linux           16118448 bytes
buzz-acp-x86_64-linux.sha256          272 bytes
buzz-x86_64-linux               16274208 bytes
git-credential-nostr-x86_64-linux  1462016 bytes
```

The `.sha256` manifest carries all three lines, including
`376c762f…  git-credential-nostr-x86_64-linux`.

## Flags for the orchestrator

1. **The verification run published a release that is now `latest=true`.**
   Triggering the workflow was the only way to satisfy acceptance 4, and
   `install-buzz-acp.sh` with no pin takes the newest `buzz-acp-linux-*` tag —
   so an unpinned install on the arch box now pulls `buzz-acp-linux-f40de5a`,
   built from this lane branch rather than from `railway-deploy`.
   Low risk, stated so it is a decision and not a surprise: this branch is
   `railway-deploy` + 3 doc-only `.cofoundy/` commits + this CI commit, so the
   binaries are functionally railway-deploy's plus the helper. To pin the deploy
   path back: `gh release delete buzz-acp-linux-f40de5a --repo cofoundy/buzz`,
   or install with an explicit sha. `railway-deploy` was not touched, rebased,
   or force-pushed.

2. **`--help` on the helper is a no-op, by design.** It has no clap; an unknown
   argument hits `lib.rs:155` (`Some(_) => return 0`) and exits 0 with zero
   output — confirmed locally. The spec asked for `--help` and it does deliver
   the signal the step wants (the binary loads, links and runs); `get` would
   block on stdin. Both the workflow and the installer carry a comment saying
   so, so nobody later reads the silent output as breakage.

3. **Deviation (small, deliberate):** the installer does not hard-fail when a
   pinned *older* build has no helper asset. Pinning an old sha is documented
   usage, and `install` would have died with `cannot stat`; it now warns and
   names the consequence instead. Any build from this commit forward installs
   all three.

4. **Untested by this lane, by design:** that a third party following only this
   README can push. That is T-003 / AM-4 and needs a credential only the
   operator unlocks. No credential was generated, minted or rotated here.

5. `.cofoundy/state/history.jsonl` accumulated hook-generated `scope_advisory` /
   `cto_gate_allow_orchestrator` lines during this run. It is a shared surface,
   so it was kept out of the deliverable commit to avoid a merge collision with
   the `cli` lane; only the `task_completed` line is committed, separately.

# DRAFT — comentario para block/buzz#2876

**Estado:** NO PUBLICADO. Requiere (a) el gate de Phase 5 confirmando que esta acción
es del CTO y no de una lane, y (b) habilitación del operador — es un acto público en
el repo de un tercero, bajo la identidad `A-PachecoT`.

**Restricciones que este texto respeta:**
- IC-3: **un solo comentario**. Si genera debate entre PRs o intercambio con
  mantenedores de Block, eso excede la delegación y vuelve a gate humano.
- A5: publica **evidencia** (reproducción + propiedades + test discriminante), no un
  veredicto sobre el PR de ningún contribuidor. No nombramos un ganador — eso sería
  arbitrar entre terceros.
- Refute-pass: **ninguna afirmación sobre el CI de ningún PR.** Nuestra lectura previa
  de `statusCheckRollup` fue no-señal leída como aprobación; no exportamos ese error.

---

We hit this independently on a self-hosted relay and ended up reproducing it from
scratch before finding this issue. Since there are now several proposed fixes open,
here is the evidence we gathered, framed as the properties any fix has to satisfy
rather than a vote for a particular patch.

### Reproduction

Against a live relay, with a repo announced more than 15 minutes earlier:

```
$ buzz repos protect set --id <repo> --ref refs/heads/main --push owner --no-force-push
{"error":"relay_error","message":"... event timestamp too far from server time"}
```

The failure is permanent, not transient: `build_updated_repo_announcement`
(`crates/buzz-cli/src/commands/repos.rs`) stamps every mutation with
`existing.created_at + 1`, while the relay validates against wall clock with a ±900s
window (`MAX_TIMESTAMP_DRIFT_SECS`, `crates/buzz-relay/src/handlers/ingest.rs`). A +1
advance never catches up to the present, so once an announcement is older than the
drift window its metadata is frozen for good — `repos bind`, `repos protect set` and
`repos protect remove` all stop being accepted.

### Why it bites harder than it looks

Combined with #2877 (the `buzz-channel` tag is the git ACL) and #2326 (no way to
delete a repo announcement), a repo announced without `--channel` becomes permanently
single-pusher with no in-band recovery. There is exactly one ~15-minute window at
creation time in which the binding can be made correct, forever. Our operational rule
while this is open is "always pass `--channel` in the same command as
`repos create`" — which works, but it is a rule humans have to remember rather than a
property the tool enforces.

### The two properties, and the test that distinguishes them

A fix has to preserve both of these **simultaneously**. This is the part worth
pinning down, because it is what separates the proposed patches:

| Property | What protects it | Case that exercises it |
|---|---|---|
| **Monotonicity** — a delayed writer must not leapfrog an intervening update and silently erase metadata (the property the original `+1` comment was defending) | the `head + 1` floor | a head in the **future** (peer clock ahead, still inside the drift window) must yield exactly `head + 1`, **not** `now` |
| **Freshness** — the relay accepts the event | the `now` floor | a head from 2 hours ago must yield a timestamp whose drift from wall clock is ≤ 900s |

A plain wall-clock stamp satisfies the second and loses the first. `head + 1` satisfies
the first and loses the second. The monotonicity case is the one we would suggest any
fix carry a regression test for, since it is the property that fails silently — a lost
`max()` floor does not produce an error, it produces an update that quietly loses to
the head it was derived from.

One adjacent note for whoever picks this up: the comment currently above that code
argues explicitly *against* using wall-clock time. Whatever fix lands, that comment
needs to be rewritten to name both floors — left as-is it reads as a prohibition on
what the corrected code does two lines below, which is plausibly part of why this
survived review in the first place.

Happy to test a candidate patch against our relay if that is useful.

# Buzz CLI usable en caja limpia — sprint `buzz-cli-usable`

**Fecha:** 2026-08-04 · **Rama:** `sprint/buzz-cli-usable` (base `railway-deploy`, nunca `main`)
**Issues:** [cofoundy/buzz#1](https://github.com/cofoundy/buzz/issues/1), [cofoundy/buzz#2](https://github.com/cofoundy/buzz/issues/2) · **Upstream:** [block/buzz#2876](https://github.com/block/buzz/issues/2876)

---

## Resultado

Dos defectos bloqueaban usar Buzz desde una caja del equipo. Los dos están
arreglados en la rama y verificados. **El ciclo no está cerrado**: su criterio de
cierre es evidencia contra el relay vivo, y eso espera un gesto del operador.

| | Antes | Ahora |
|---|---|---|
| `git push` contra el relay desde caja limpia | imposible — el helper NIP-98 no se shippeaba; había que compilarlo (1m35s + toolchain Rust) | `git-credential-nostr` sale como tercer asset del release y lo instala `install-buzz-acp.sh` |
| `repos bind` / `repos protect` sobre repo de >15 min | rechazado para siempre — `event timestamp too far from server time` | `max(head + 1, now)`: monotonía y frescura preservadas a la vez |
| Cambio en el crate del helper | no redisparaba el build (`paths:` lo omitía) | dispara |
| Config de push documentada | no existía `deploy/arch-box/README.md` | existe, con cada string de error reproducido contra un binario real |

## Evidencia

Todo lo de abajo lo corrió el CTO de forma independiente, no se aceptó del reporte
de la lane.

**Release `buzz-acp-linux-f40de5a`** — descargado y verificado contra su manifiesto:

```
buzz-acp-x86_64-linux: OK
buzz-x86_64-linux: OK
git-credential-nostr-x86_64-linux: OK      ← el asset nuevo
```

Corrida [30936986998](https://github.com/cofoundy/buzz/actions/runs/30936986998): los
10 pasos en verde, incluido el smoke check de los tres binarios.

**Tests sobre el árbol mergeado**, no sobre las lanes:

```
cargo test -p buzz-cli   → 273 passed; 0 failed
cargo clippy -p buzz-cli -- -D warnings → limpio
cargo fmt --check → limpio
```

Los dos tests de propiedad corren por nombre:
`update_of_fresh_head_stays_monotonic_over_it` y
`updated_announcement_stays_within_relay_drift_window_for_stale_head`.

**Los strings de error del README existen en el fuente** (`lib.rs:35`, `:57`, `:197`) —
documentación fundamentada, no transcrita de memoria.

## Las dos propiedades del fix de #2

El defecto: `build_updated_repo_announcement` firmaba con `existing.created_at + 1`,
mientras el relay valida contra reloj de pared con ventana ±900s. El +1 nunca alcanza
al presente, así que a los 15 minutos los metadatos quedan congelados **para siempre**.

Cualquier fix debe preservar dos propiedades **a la vez**:

| Propiedad | Piso que la protege | Caso que la ejercita |
|---|---|---|
| **Monotonía** — un writer demorado no pisa una edición intermedia | `head + 1` | head en el futuro ⇒ exactamente `head + 1`, no `now` |
| **Frescura** — el relay acepta el evento | `now` | head de 2 h ⇒ drift ≤ 900s |

La lane no asumió que el fix tuviera dientes: **lo mutó en ambas direcciones**.

| variante | monotonía | frescura | resultado |
|---|---|---|---|
| `head_floor` (el bug original) | ok | falla | 3 tests fallan |
| `Timestamp::now()` pelado | falla | ok | 1 test falla |
| `head_floor.max(now)` (el fix) | ok | ok | 273/0 |

Eso importó particularmente acá — ver abajo.

## Dos decisiones que valen más que el código

**No escribimos el fix de #2. Lo portamos.** Verificamos que ya hay **tres PRs
abiertos upstream** para el mismo bug. Un cuarto habría sido ruido. Portamos la
semántica de #4363/#4509 citando ambos SHAs, y en vez de un PR duplicado publicamos
**evidencia** en el issue: la reproducción, las dos propiedades, y el test que las
distingue. Sin nombrar un PR ganador — eso sería arbitrar entre terceros.

**El "CI verde" de esos PRs era no-señal.** La primera lectura de este ciclo afirmó
que #4363 y #4509 tenían CI verde, parseando `statusCheckRollup`. Un refute-pass
adversarial lo refutó y `gh pr checks` lo confirmó:

| PR | Lo que realmente corrió |
|---|---|
| #4363, #4509 | DCO, Semgrep, zizmor — **cero builds de Rust, cero tests** |
| #2901 | Build amd64/arm64, relay e2e, Desktop — todos pass; falla solo DCO signoff y Security |

Es el misread que `handbook/governance/git-strategy.md` ya pre-registra (inbox-ai#278):
un array de checks trivial se lee como "sin fallos" cuando significa **"sin señal"**.
Consecuencia concreta: los tests de #4363 nunca se compilaron y contienen asserts
vacuos — `>= 1000` contra un fixture de `100`, `>= 101` contra un reloj de ~1.7e9.
Se descartaron. El rechazo de #2901 se mantiene, pero por perder la monotonía, no por
su CI.

## Lo que falta — un gesto, no una sesión

El criterio de cierre del brief es *"evidencia ejecutada contra `buzz.cofoundy.dev`,
no compilación verde"*. Vaultwarden está locked, así que las tres evidencias vivas
(`T-003`) esperan:

```bash
bw unlock     # + exportar BUZZ_PRIVATE_KEY / NOSTR_PRIVATE_KEY
```

Con eso corren solas: (E-1) `repos protect set` sobre un repo de >15 min devolviendo
`accepted:true`, (E-2) el helper presente en `~/.local/bin` de una caja sin toolchain,
(E-3) `clone` + `push` verdes siguiendo **solo** el README — el único test real del
README, porque T-001 verifica que el archivo existe y documenta, nunca que un tercero
siguiéndolo logre pushear.

Detrás de eso se secuencia el comentario a block/buzz#2876, redactado y **sin
publicar**: su aporte diferencial es justamente la reproducción contra relay vivo.
Es un acto público e irreversible en el repo de un tercero — requiere tu autorización.

## Riesgo abierto

La corrida de verificación publicó un release desde una rama de lane, que quedó
`latest=true`, y `install-buzz-acp.sh` sin pin toma el más nuevo. **Se dejó a
propósito:** cero código de crate difiere de `railway-deploy` (`a67d5d0`), así que
`buzz-acp` y `buzz` están construidos de fuente idéntica y el release es un superset
estricto. El workflow solo auto-dispara sobre `railway-deploy`, así que la operación
normal no lo repite.

## Referencias

- Rama: `sprint/buzz-cli-usable` @ `40d266cb5` (14 commits sobre `railway-deploy`) · PR [#3](https://github.com/cofoundy/buzz/pull/3) (merged)
- Substrate del ciclo: `.cofoundy/` — decisiones de gate, arquitectura, tasks, reportes por lane
- Borrador upstream: `.cofoundy/state/upstream-2876-comment.draft.md`

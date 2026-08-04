# Lane `cli` — T-002: descongelar los metadatos de repo

**Branch:** `lane/cli-timestamp-unfreeze` (base `sprint/buzz-cli-usable`)
**Issue:** cofoundy/buzz#2 · **Upstream:** block/buzz#2876
**Archivo tocado:** `crates/buzz-cli/src/commands/repos.rs` (único en `scope.write` de código)

---

## Qué se portó y de dónde

El fix es `next_created_at = max(head + 1, now)`. Se portó la **semántica** de los dos
PRs upstream, no el texto — como manda la tarea, porque no hay hunk común literal.

| PR | SHA de la rama | Qué aporta | Qué se tomó |
|---|---|---|---|
| [#4363](https://github.com/block/buzz/pull/4363) | `Timestamp::now().as_secs().max(head_floor)` | el test que nombra el bound ±900 | `updated_announcement_stays_within_relay_drift_window_for_stale_head` |
| [#4509](https://github.com/block/buzz/pull/4509) | `bumped_head.max(Timestamp::now().as_secs())` | el test de monotonía + asserts ajustados | `update_of_fresh_head_stays_monotonic_over_it`, patrón `>= before && <= after` |
| [#2901](https://github.com/block/buzz/pull/2901) | reloj de pared a secas | — | **descartado**: pierde la monotonía (demostrado abajo, Mutación 2) |

Verificado leyendo ambos diffs: difieren **solo** en nombre del binding
(`head_floor` vs `bumped_head`) y en el comentario. Semánticamente idénticos —
`a.max(b)` es conmutativo. No hubo que desempatar por la tabla de propiedades.
Se adoptó el orden de #4509 (`head_floor.max(now)`) con nombre de #4363
(`head_floor`) y comentario propio (R-4).

### Qué se descartó explícitamente

- **Los asserts vacuos de #4363**, tal como los describe la tarea. Confirmado leyendo el
  diff: `assert!(created_at >= 1000)` contra un fixture de `100` y
  `assert!(created_at >= 101)` — con el fix `created_at ≈ 1.7e9`, así que ambos pasan
  siempre y no prueban nada.
- **El test `protection_set_preserves_metadata_and_protections` de #4363** entero: duplica
  la cobertura de tags del test que ya existe en nuestro árbol
  (`protection_update_preserves_metadata_and_replaces_only_matching_pattern`) y su único
  aporte nuevo eran los dos asserts vacuos de arriba.

### Desviación consciente (una)

En el sitio `:717` (`bind_channel_replaces_duplicates…`) **#4509 usa
`>= existing.created_at.as_secs()`**, o sea `>= 100` contra un reloj de ~1.7e9 — vacuo
por el mismo motivo que los de #4363. Se aplicó ahí el bracket ajustado
`>= before && <= after` en vez de portarlo literal. Es más estricto que cualquiera de los
dos PRs y sigue la directiva de la tarea: assertear las propiedades, **nunca** un literal.

---

## R-3 y R-4

- **R-3** — los dos `assert_eq!(updated.created_at.as_secs(), 101)` (`:523`, `:717`)
  eliminados. Ambos pasan a assertear la propiedad de frescura con el bracket
  `before`/`after` que envuelve la llamada. Cero literales de timestamp en el archivo.
- **R-4** — el comentario de `:144` ya no argumenta contra el reloj de pared. Ahora nombra
  las **dos** propiedades y **cuál piso protege cuál**: `head + 1` → monotonía (piso
  vinculante con head futuro), `now` → frescura (piso vinculante con head stale), y por qué
  cada uno solo es un bug distinto.

---

## Las dos propiedades están enforced por separado (mutation testing)

El riesgo central de esta tarea es shipear tests que pasan sin probar nada. Se verificó
mutando el fix en las dos direcciones y confirmando qué falla:

| Variante de `next_created_at` | monotonía | frescura (3 tests) | total |
|---|---|---|---|
| `head_floor` — bug original | ok | **FAILED** | 270 passed, **3 failed** |
| `Timestamp::now()` — error de #2901 | **FAILED** | ok | 272 passed, **1 failed** |
| `head_floor.max(now)` — **el fix** | ok | ok | **273 passed, 0 failed** |

Cada propiedad tiene al menos un test que falla cuando —y solo cuando— esa propiedad se
rompe. Ninguno de los dos es vacuo, y la tabla reproduce exactamente el argumento de la
tarea sobre por qué `now` a secas es incorrecto.

---

## Aceptación — output real

### 1. `cargo test -p buzz-cli` — verde

```
test commands::repos::tests::update_of_fresh_head_stays_monotonic_over_it ... ok
test commands::repos::tests::updated_announcement_stays_within_relay_drift_window_for_stale_head ... ok
test commands::repos::tests::protection_update_preserves_metadata_and_replaces_only_matching_pattern ... ok
test commands::repos::tests::bind_channel_replaces_duplicates_and_preserves_everything_else ... ok

test result: ok. 273 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s

     Running unittests src/main.rs (target/debug/deps/buzz-d4a8c93353fd965f)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests buzz_cli
running 1 test
test crates/buzz-cli/src/lib.rs - run_from_args (line 19) ... ignored
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Los dos tests de propiedad presentes; los dos asserts literales eliminados.

### 2. `cargo clippy -p buzz-cli -- -D warnings` — limpio

```
    Checking buzz-ws-client v0.1.0 (/Users/.../crates/buzz-ws-client)
    Checking buzz-cli v0.1.0 (/Users/.../crates/buzz-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 37.47s
```

Cero warnings con `-D warnings`.

### 3. `cargo fmt --check` — limpio

```
(sin output — exit 0)
```

---

## Notas para el CTO

- **Base drift al spawn (corregido).** El worktree nació en `b1b283cd4` = `main`, no en
  `sprint/buzz-cli-usable`; `.cofoundy/` ni siquiera existía. Se aplicó el spawn step 0 del
  agent-floor: `git checkout -B lane/cli-timestamp-unfreeze sprint/buzz-cli-usable`, luego
  `merge-base --is-ancestor` OK. Vale la pena revisar por qué el harness lo creó off `main`
  — es exactamente el fallo que el floor §2026-07-22-worktree-base-drift anticipa, y otras
  lanes de este sprint arrancarían igual.
- **Evidencia contra relay vivo: NO incluida**, según la tarea (es de T-003, gate de Phase 5
  AM-2, depende de credencial que solo el operador desbloquea). No se generó, minteó ni rotó
  ninguna credencial. Vaultwarden sigue locked y no se tocó.
- **`ingest.rs` solo lectura** — `MAX_TIMESTAMP_DRIFT_SECS` no se tocó. El valor `900` se
  duplica como `const` local en el test nuevo solo para nombrar el bound en el mensaje de
  fallo; no importa ni modifica el del relay.
- **Riesgo forward-only** (ya en el spec §"Riesgo sobre datos vivos"): revertir el binario no
  revierte timestamps ya emitidos, y un cliente con lógica vieja (`head + 1`) queda
  permanentemente por debajo en un LWW contra un repo que nosotros toquemos. Inocuo en
  piloto founders-only con un operador; deja de serlo con >1 cliente escribiendo metadatos.

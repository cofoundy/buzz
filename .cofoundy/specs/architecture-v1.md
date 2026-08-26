# architecture-v1 — buzz-cli-usable

**Ciclo:** 2026-08-04 · **Base:** `origin/railway-deploy` · **Rama:** `sprint/buzz-cli-usable`

> Nota de proceso: se omitió el dispatch de Plan-agent (Phase 2b). El espacio de
> diseño de #2 está enumerado exhaustivamente por tres implementaciones upstream
> ya escritas y leídas durante el recon; el de #1 es una adición mecánica a una
> matriz de build. Un Plan-agent habría reformulado lo que los PRs ya dicen. El
> gate de ceo-agent sí corre — ahí está el chequeo real.

## Estado verificado (no inferido)

| Aserción | Verificación | Resultado |
|---|---|---|
| block/buzz#2876 sigue abierto | `gh issue view 2876` | OPEN, updated 2026-08-02 |
| upstream `main` sigue con el bug | `git show origin/main:…/repos.rs` | sí, `checked_add(1)` intacto |
| existen PRs upstream para el bug | `gh pr list --search 2876` | **tres**, todos OPEN |
| qué CI corrió realmente en cada PR | `gh pr checks <pr>` | ver abajo — **no** `statusCheckRollup` |

### Corrección del refute-pass: "CI verde" era no-señal

La pasada 1 de este doc afirmó que #4363 y #4509 tenían "CI verde", leyendo
`statusCheckRollup` con `jq`. El refute-pass F1c lo refutó y `gh pr checks` lo confirma:

| PR | Checks que corrieron | Lectura correcta |
|---|---|---|
| #4363 | DCO, Semgrep, zizmor | **cero builds de Rust, cero tests** — su código nunca compiló upstream |
| #4509 | DCO, Semgrep, zizmor | ídem |
| #2901 | Build amd64+arm64, relay e2e, Desktop… todos **pass**; fallan solo DCO signoff y Security | es el único cuyo código Rust upstream sí compiló |

Es exactamente el misread que `handbook/governance/git-strategy.md` pre-registra
(inbox-ai#278): un array de rollup vacío o trivial se lee como "sin fallos" cuando
significa **"sin señal"**. Consecuencias vinculantes:

- **"CI verde" queda anulado como garantía de calidad.** Nuestro propio
  `cargo test -p buzz-cli` es el **único** gate sobre el código portado.
- Los tests de #4363 nunca se compilaron y contienen asserts vacuos
  (`>= 1000` contra un fixture de 100; `>= 101` contra reloj de pared ~1.7e9).
  **No se adoptan verbatim** — ver §"Cuál portar".
- El rechazo de #2901 **se mantiene**, pero por su razón sustantiva (pierde la
  monotonía), no por su CI. Esa razón es correcta con independencia del CI.

## Las dos lanes

### Lane `ci` — issue #1: shippear `git-credential-nostr`

Puramente aditiva. El workflow ya construye dos binarios (`buzz-acp`, `buzz-cli`)
y publica assets + sha256 a un release por commit. Agregar un tercero es extender
tres listas que ya existen, más el instalador que las consume.

```
.github/workflows/buzz-acp-linux.yml
  paths:        + crates/git-credential-nostr/**   ← R-2: hoy un cambio al helper no redispara
  cargo build:  + -p git-credential-nostr
  smoke check:  + --help del helper                 ← no shippear un binario que no arranca
  assets:       + git-credential-nostr-x86_64-linux (entra al sha256sum existente)

deploy/arch-box/install-buzz-acp.sh
  + HELPER_ASSET + install -m 0755 → $DEST/git-credential-nostr

deploy/arch-box/README.md                           ← R-1: NO EXISTE, se crea
  los dos requisitos de config no obvios que hacen fallar el push:
    · git config credential.useHttpPath true  (obligatorio — el helper aborta sin esto)
    · NOSTR_PRIVATE_KEY en el entorno, o git config nostr.keyfile <path> con 0600
```

El nombre del release tag (`buzz-acp-linux-<sha>`) no cambia: el instalador ya
filtra por ese namespace y renombrarlo rompería los pins existentes.

### Lane `cli` — issue #2: metadatos congelados

El fix es conocido y está escrito tres veces upstream. **No escribimos un cuarto.**

`build_updated_repo_announcement` (`crates/buzz-cli/src/commands/repos.rs:146`)
firma con `existing.created_at + 1`. El relay valida contra reloj de pared con
ventana ±900s (`MAX_TIMESTAMP_DRIFT_SECS`). Como el avance es de +1 y nunca
alcanza al presente, pasados 15 min los metadatos quedan congelados para siempre.

**Fix:** `next_created_at = max(existing.created_at + 1, now)`.

Las dos propiedades que el fix debe preservar simultáneamente — y que son la razón
de que `now` a secas (PR #2901) sea incorrecto:

| Propiedad | Qué la protege | Caso que la ejercita |
|---|---|---|
| **Monotonía** — un writer demorado no pisa una edición intermedia | el piso `head + 1` | head en el **futuro** (reloj del peer adelantado) → resultado debe ser `head + 1`, no `now` |
| **Frescura** — el relay acepta el evento | el piso `now` | head de hace 2 h → drift resultante vs. reloj de pared ≤ 900s |

R-3: `repos.rs:523` y `:717` assertean `created_at == 101` exacto. Con el fix eso
deja de ser cierto — pasan a assertear las dos propiedades de arriba, no un literal.

**R-4 (hallado en el gate):** el comentario en `repos.rs:144-145` argumenta
explícitamente *en contra* del reloj de pared ("Using wall-clock time here would let
a delayed writer leapfrog…"). Tras el fix quedaría prohibiendo lo que el código hace
dos líneas abajo. Debe reescribirse para nombrar las dos propiedades y cuál piso
protege cuál — plausiblemente ese comentario es parte de por qué el bug sobrevivió
review upstream. Ambos PRs portables ya lo reescriben; verificar que el port lo traiga.

### Riesgo sobre datos vivos (no sobre el relay)

El relay no se toca — el radio de esta lane es nuestra caja y los repos que ya
anunciamos. Dos efectos que el cierre debe asumir conscientemente:

- **Cross-client:** tras el fix, un writer que todavía corra la lógica vieja
  (`head + 1`) no puede volver a ganar un write NIP-33 LWW contra un repo que
  nosotros hayamos tocado — su timestamp queda permanentemente por debajo. En un
  piloto founders-only con un solo operador esto es inocuo, pero deja de serlo
  cuando haya más de un cliente escribiendo metadatos.
- **Rollback asimétrico:** revertir el binario no revierte los timestamps ya
  emitidos. El fix es forward-only en la práctica.

## La decisión: fork, upstream, o ambos

El operador delegó esto explícitamente. El estado verificado la reencuadra: con
**tres** PRs abiertos, dos de ellos con exactamente nuestro enfoque y CI verde, un
cuarto PR es ruido, no ciudadanía OSS.

**Propuesta: portar + endosar, no duplicar.**

1. **Fork:** portar el fix de los PRs upstream existentes. Lo queremos ya — la
   protección de ramas mutable es lo que desbloquea repos reales en Buzz.

   **Por qué portar y no escribir el nuestro — justificación corregida.** La pasada 1
   apoyó el port en el ahorro de rebase (patch-id equivalence descartaría nuestro
   commit cuando upstream mergee). El refute-pass demostró que ese beneficio es
   **estructuralmente cero, no "probable"**: adoptar tests de ambos PRs y escribir
   nuestro propio comentario en el mismo hunk garantizan, cada uno por separado, que
   nuestro diff no coincida con ningún patch-id upstream. Dos de las enmiendas de este
   mismo doc matan el beneficio en que descansaba su justificación.

   Lo que sostiene el port es **procedencia y convergencia**, no ahorro mecánico:
   partimos de código que otros ya derivaron del mismo diagnóstico, convergemos al
   texto hacia el que upstream va a converger, y no inventamos una tercera variante de
   un fix que ya tiene tres. Registrar el SHA upstream portado en el cuerpo del commit
   — eso es lo que hace auditable el rebase futuro, con o sin patch-id.
2. **Upstream:** ningún cuarto PR — **ratificado**. En su lugar publicamos
   **evidencia** en #2876: nuestra reproducción independiente contra un relay vivo,
   las dos propiedades que cualquier fix debe preservar simultáneamente, y el caso de
   test que las distingue. Un issue estancado con tres PRs sin review no necesita un
   cuarto; necesita el material con el que un reviewer pueda decidir. Publicar
   propiedades y un test discriminante — no un veredicto nominal sobre el PR de un
   contribuidor — es lo que le sirve al mantenedor y lo que evita que estemos
   arbitrando entre terceros.

**Cuál portar: la pregunta se disuelve, pero no hay "hunk común" textual.** El
refute-pass verificó los dos diffs: #4363 escribe
`Timestamp::now().as_secs().max(head_floor)`, #4509 escribe
`bumped_head.max(Timestamp::now().as_secs())`. **Semántica idéntica, texto distinto**
(distinto nombre de binding, distinto comentario). No existe un hunk común que copiar
literalmente — se porta la *semántica* `max(head + 1, now)`, con nuestro propio
comentario (R-4), citando ambos SHAs.

Los tests **no se adoptan verbatim** — nunca fueron compilados upstream (§corrección
del refute-pass). Se toma lo que cada uno aporta y se descarta lo vacuo:

| De | Tomar | Descartar |
|---|---|---|
| #4509 | el test de monotonía (head futuro ⇒ exactamente `head + 1`) y los asserts ajustados `>= before && <= after` | — |
| #4363 | `updated_announcement_stays_within_relay_drift_window_for_stale_head` — el único que nombra el bound ±900 | los asserts vacuos `>= 1000` (fixture de 100) y `>= 101` (reloj de pared ~1.7e9): pasan siempre, no prueban nada |

Los dos tests que quedan son **complementarios, no competidores**: uno por propiedad
de la tabla de arriba. El doc ya no elige entre PRs con un criterio ("cuál mergea")
que es indecidible hoy y lo seguiría siendo en implementación.

## Guardrails heredados de los issues (no negociables)

- No tocar el Dockerfile del relay ni `deploy/railway/`.
- No ampliar `MAX_TIMESTAMP_DRIFT_SECS` — la ventana es defensa anti-replay, el defecto es del CLI.
- No modificar el crate `git-credential-nostr` (funciona; verificado con push y clone reales).
- No rebasear ni forzar sobre `railway-deploy`.

## Criterio de cierre

Evidencia ejecutada contra `buzz.cofoundy.dev`, no compilación verde:

- `git-credential-nostr` en `~/.local/bin` de una caja sin toolchain, vía instalador.
- `git push` verde contra el relay siguiendo solo el README.
- `repos protect set` sobre un repo anunciado hace >15 min → `accepted: true`.

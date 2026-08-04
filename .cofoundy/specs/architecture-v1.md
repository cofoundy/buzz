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

1. **Fork:** portar el hunk de producción de los PRs upstream existentes. Lo queremos
   ya — la protección de ramas mutable es lo que desbloquea repos reales en Buzz.
   "Verbatim" aplica **solo al hunk de producción, nunca a los tests.** El beneficio
   es que si upstream mergea ese texto sin cambios, `git` descarta nuestro commit por
   equivalencia de patch-id en el rebase; una variante propia garantiza conflicto.
   Eso es *probabilidad, no propiedad* — si el PR muta antes de mergear, el conflicto
   vuelve. Registrar en el cuerpo del commit el SHA upstream portado, para que el
   rebase futuro sea auditable.
2. **Upstream:** ningún cuarto PR — **ratificado**. En su lugar publicamos
   **evidencia** en #2876: nuestra reproducción independiente contra un relay vivo,
   las dos propiedades que cualquier fix debe preservar simultáneamente, y el caso de
   test que las distingue. Un issue estancado con tres PRs sin review no necesita un
   cuarto; necesita el material con el que un reviewer pueda decidir. Publicar
   propiedades y un test discriminante — no un veredicto nominal sobre el PR de un
   contribuidor — es lo que le sirve al mantenedor y lo que evita que estemos
   arbitrando entre terceros.

**Cuál portar: la pregunta se disuelve.** #4363 y #4509 son ambos
`max(now, head + 1)` con CI verde: la línea de producción es casi con certeza el
mismo texto. Difieren en los tests — que reescribimos igual por R-3 — y esos tests
son **complementarios, no competidores**: #4509 ejercita la monotonía (head en el
futuro ⇒ `head + 1`), #4363 ejercita la frescura (head de 2 h ⇒ drift ≤ 900s). Son
exactamente las dos propiedades de la tabla de arriba, una cada uno.

→ **Portar el hunk de producción común y adoptar AMBOS casos de test.** El doc ya no
elige entre PRs con un criterio ("cuál mergea") que es indecidible hoy y lo seguirá
siendo en implementación.

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

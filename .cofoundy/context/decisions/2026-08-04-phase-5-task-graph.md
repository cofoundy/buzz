# Decision: task graph T-001/T-002 — enmendar antes de despachar; upstream sale del grafo, evidencia viva se carva a T-003

**Phase:** 5
**Date:** 2026-08-04
**Authority:** ceo-agent (tier-1 partner delegation, /cto cycle `buzz-cli-usable` / 2026-08-04)
**Status:** amended

## Question

/cto pide resolver el gate de Phase 5 sobre `.cofoundy/tasks/T-001.md`, `.cofoundy/tasks/T-002.md` y
`.cofoundy/specs/file-ownership-matrix.md`. Contratos a validar: (a) toda línea de aceptación
testeable, (b) ninguna celda de la matriz con 2+ writers, (c) dependencias en DAG, (d) ningún scope
contradice `brief.yaml:mvp_scope`. Más dos preguntas puntuales: si `block/buzz` pertenece al grafo, y
si el vault bloqueado hace el grafo inaprobable.

## Decision

**Enmendar, no despachar todavía.** Los cuatro contratos de Phase 5 pasan — el grafo es sólido y las
tres IC de Phase 2 aterrizaron. Pero hay **cuatro enmiendas**, y una de ellas (AM-3) es un hueco de
seguridad que por sí solo justifica no despachar así: nada en el grafo le prohíbe a un worker
**generar credenciales** cuando descubra que no las tiene.

Las dos preguntas se responden: **el comentario upstream sale del grafo** (pasa a acción de CTO), y
**el vault bloqueado NO hace el grafo inaprobable** — se carva la evidencia viva a un T-003
operator-gated. Consecuencia de la primera: el grafo baja a un repo y `task_graph_repos_gt` deja de
disparar (ver §Blast radius — lo declaro con su condicional, no como conveniencia).

## Validación de los cuatro contratos

**(a) Toda línea de aceptación testeable — PASA, con un hueco de cobertura.**

T-001: las cuatro son verificables (1-3 por lectura de archivo, 4 por `gh release view`). Nota de
mérito: la línea 4 pre-registra el modo de falla — *"verificado con `gh release view`, no asumido del
log del job"*. Es exactamente la clase de misread que el refute-pass acaba de encontrar en el CI
upstream; verlo internalizado en el grafo es buena señal.

T-002: las cuatro testeables. 1-3 son `cargo test/clippy/fmt` + lectura. La 4 es evidencia viva
(bloqueada — ver AM-2).

**El hueco:** `mvp_scope` ítem 2 dice *"una caja sin toolchain de Rust puede clonar y pushear
siguiendo solo el README"*. La aceptación de T-001 verifica que el README **existe y documenta** los
dos requisitos (línea 3) y que el asset **se publica** (línea 4). Nunca verifica que un tercero
siguiendo solo ese README **logre pushear**. Un README puede ser correcto por archivo y aun así
insuficiente por omisión. T-001 puede darse por hecho sin satisfacer el ítem de scope que lo motiva.
→ **AM-4.**

Corrijo de paso el encuadre que me pasó /cto: la última línea de aceptación de **T-001 no** es
evidencia contra el relay vivo — es `gh release view`. El bloqueo por vault en la lane `ci` no está
en una línea presente, está en la **línea ausente**. Eso cambia dónde hay que remediar.

**(b) Ninguna celda con 2+ writers — PASA.**

Verifiqué celda por celda y crucé cada `scope.write` de tarea contra su columna:

| Tarea | `scope.write` | Celda de matriz | ¿Consistente? |
|---|---|---|---|
| T-001 | workflow, install script, README, `reports/ci.md` | `ci: W` en las cuatro | ✅ |
| T-002 | `repos.rs`, `reports/cli.md` | `cli: W` en ambas | ✅ |

Intersección de los dos conjuntos de escritura: **vacía**. Las lanes son disjuntas por construcción
(`ci` en `.github/` + `deploy/arch-box/`, `cli` en un solo archivo Rust) ⇒ paralelizables sin rama de
integración, como afirma la matriz. Los tests de T-002 viven en el mismo `repos.rs` (`#[cfg(test)]`
in-file, verificado: `:523` y `:717`), así que no hace falta ampliar su scope de escritura.
`.cofoundy/state/reports/<role>.md` aparece con `W` en ambas columnas pero es un glob parametrizado
por rol — un archivo por lane, colisión imposible. Notación algo laxa, semántica correcta.

**(c) DAG — PASA.** `T-001.blockedBy: []`, `T-002.blockedBy: []`. Dos nodos independientes; DAG
trivial, sin ciclos. Con el T-003 de AM-2 (`blockedBy: [T-001, T-002]`) sigue siendo DAG.

**(d) Ningún scope contradice `mvp_scope` — PASA.** Cobertura: ítem 1 → T-001 ✅; ítem 2 → T-001
**parcial** (hueco de AM-4); ítem 3 → T-002 aceptación 4 (bloqueada); ítem 4 → T-002 aceptación 1 ✅.
Ninguna tarea propone nada **fuera** de `mvp_scope`, así que `thresholds.scope_expansion_request` —
que sí aplica a Phase 5 — **no dispara**. Los guardrails de `out_of_scope` están replicados en ambas
tareas y en la matriz (relay/Dockerfile/railway en solo-lectura, `MAX_TIMESTAMP_DRIFT_SECS` intocable,
sin rebase sobre `railway-deploy`).

## Las tres IC de Phase 2

| IC | ¿Aterrizó? | Dónde |
|---|---|---|
| **IC-1** — "hunk común" es premisa a verificar; si difieren, desempata la tabla de propiedades y se registra el descarte | ✅ **y ya disparó** | T-002 L37-40 (*"No hay un 'hunk común' que copiar literalmente"*) + L65-66 (desempate + *"registrá qué descartaste"*). La condición que puse como hipotética se cumplió: difieren en binding y comentario. IC-1 se ganó el lugar. |
| **IC-2** — R-4 se satisface aunque el port no lo traiga | ✅ vía aceptación | T-002 L82-85 mantiene *"ambos PRs ya lo reescriben, verificá que tu port lo traiga"* — condicional. Pero **aceptación 3** (*"El comentario de `:144` describe las dos propiedades y qué piso protege cada una"*) es **incondicional** y no depende del port. Esa es la forma vinculante que pedí. Satisfecho. |
| **IC-3** — un solo comentario upstream, sin arbitrar, escala si deriva en debate | ⚠️ **parcial** | T-002 L110 saca la acción de la lane (*"La acción upstream la maneja el CTO"*) ✅. Pero el **contenido** de IC-3 no está escrito en ningún artefacto del grafo — vive solo en mi decisión de Phase 2. Ejecutable por /cto, no auditable desde el grafo. → **AM-1** lo formaliza. |

**Sobre IC-2, una advertencia derivada del refute-pass:** el refuter demostró que *"CI verde en
#4363/#4509"* era no-señal leída como aprobación. La frase *"ambos PRs portables ya lo reescriben"*
(T-002 L84) pertenece a **la misma familia**: una aserción sobre esos PRs que nunca entró a la tabla
de estado verificado. Puede ser igual de falsa. No pido verificarla — la aceptación 3 la vuelve
irrelevante, que es justo por qué IC-2 se escribió incondicional. Queda como recordatorio de que las
premisas sobre PRs no compilados no se heredan.

## Amendments

**AM-1 — Sacar `block/buzz` del grafo de tareas.**
`file-ownership-matrix.md` §"Repos tocados" declara dos repos. Reescribir: **el grafo toca un repo**,
`cofoundy/buzz`. El comentario de evidencia en block/buzz#2876 es una **acción de CTO fuera de las
lanes**, gobernada por IC-3 de la decisión de Phase 2, y **secuenciada después de T-003** (su valor es
la reproducción contra relay vivo, que hoy está bloqueada). Añadir a la matriz el puntero a IC-3 para
que la restricción sea auditable desde el grafo y no solo desde mi decisión anterior.

**AM-2 — Carvar la evidencia viva a `T-003`, operator-gated.**
Mover la aceptación 4 de T-002 (y la línea nueva de AM-4) a un **T-003** nuevo:
`blockedBy: [T-001, T-002]`, `status: blocked`, precondición explícita **"requiere que el operador
desbloquee Vaultwarden (`bw unlock`) y exporte `BUZZ_PRIVATE_KEY` / `NOSTR_PRIVATE_KEY`"**. T-001 y
T-002 quedan **enteramente desbloqueadas** y despachables ya. Consecuencia que hay que decir sin
maquillar: con T-003 pendiente, **el ciclo no puede declararse cerrado** — el `Criterio de cierre` del
brief es *"evidencia ejecutada contra buzz.cofoundy.dev, no compilación verde"*. El grafo es
despachable; el ciclo no es cerrable. Son cosas distintas y no hay que confundirlas al reportar.

**AM-3 — Prohibir explícitamente mintear credenciales. (El motivo por el que no despacho aún.)**
T-002 L102-103 dice *"Si no tenés credenciales para el relay, decilo — no simules el resultado"*. Eso
prohíbe **simular**, no prohíbe **conseguir**. Un agente diligente lee "no simules" como "entonces
conseguí credenciales de verdad" y el camino obvio es generar un keypair nostr nuevo (`buzz keys
generate` o equivalente) y anunciarle un repo al relay. Eso sería
`always_escalate.credential_mint_or_rotate` — una superficie **no relajable por `autonomy_overrides`**
— disparada por un worker, dentro de un grafo que yo aprobé. Añadir a los guardrails de **ambas**
tareas, textual:

> No generes, mintees ni rotes credenciales (claves nostr, tokens, entradas de vault) bajo ninguna
> circunstancia. Si falta una credencial, la tarea se detiene y lo reporta. Conseguir credenciales es
> acción del operador, nunca de la lane.

**AM-4 — Cerrar el hueco de cobertura de `mvp_scope` ítem 2.**
Añadir a T-003 la línea que hoy no existe en ningún lado: en una caja **sin toolchain de Rust**,
siguiendo **solo** `deploy/arch-box/README.md`, un `git clone` + `git push` contra
`buzz.cofoundy.dev` completa verde. Es el único test real del README; su ausencia dejaba a T-001
declarable "hecha" sin satisfacer el ítem de scope que la motiva.

## Las dos preguntas, respondidas

### 1. ¿`block/buzz` pertenece al grafo? — No. Es acción de CTO.

Cuatro razones, la tercera decisiva:

1. **No tiene aceptación testeable.** Es un acto comunicativo; no compila, no corre, no pasa/falla.
   Meterlo en un grafo cuyo contrato es "toda línea de aceptación testeable" lo corrompe.
2. **Es irreversible y público bajo identidad de la organización**, en el repo de un tercero.
   Asimétrico respecto de todo lo demás del grafo, que es reversible con un `git revert`.
3. **El operador me delegó la decisión a mí, no a un worker.** "Vos decidís si parcheás el fork,
   upstreameás, o ambos" es delegación a este gate. Un agente de lane publicando en block/buzz es una
   delegación **estrictamente mayor** que la que se hizo, y hecha por mí, no por él. No la hago.
4. **Su contenido es gobernanza, no implementación.** Lo rige A5/IC-3 (evidencia, no veredicto; sin
   arbitrar entre terceros). Los workers no cargan gobernanza; cargan specs.

Y hay una dependencia real que el grafo no modela: A5 exige publicar *"nuestra reproducción
independiente contra un relay vivo"*. Sin T-003 no tenemos esa reproducción — **el comentario upstream
está bloqueado por el mismo candado de vault**. Publicarlo antes sería publicar sin la evidencia que
es justamente nuestro aporte diferencial. Secuencia correcta: T-001 ∥ T-002 → T-003 → acción CTO
upstream.

### 2. ¿El vault bloqueado hace el grafo inaprobable? — No. Aprobable con la evidencia viva diferida.

El bloqueo es **de credencial, no de diseño**. Nada en el grafo está mal; falta una llave que solo el
operador tiene. Bloquear el dispatch entero por eso desperdiciaría toda la ventana de autonomía en un
candado que nadie más puede abrir — exactamente el desperdicio que el contrato de autonomía nombra
("parar ante una pregunta desperdicia la ventana"). Y lo que queda desbloqueado no es marginal: el
fix, sus tests de propiedad, clippy/fmt, el workflow de CI, el instalador, el README y el release
son la mayor parte del valor del ciclo, y todos verificables sin tocar el relay.

Lo que **no** hago es fingir que eso cierra el ciclo. El brief pone la vara en evidencia ejecutada
contra `buzz.cofoundy.dev`; con T-003 pendiente el ciclo queda en *"todo verificable hecho y
verificado; falta la verificación viva, gated en el operador"*. Ese es el punto de resume limpio que
el contrato de autonomía sí exige: rama commiteada, lanes cerradas, un solo gesto humano pendiente y
nombrado.

**Qué necesita Andre a la vuelta** — un gesto, no una sesión:

```bash
bw unlock                                     # y exportar BUZZ_PRIVATE_KEY / NOSTR_PRIVATE_KEY
```

Con eso T-003 corre solo y arrastra el comentario upstream detrás.

## Alternatives considered

- **Aprobar y despachar tal cual** — rechazada por AM-3. Despachar un grafo que no prohíbe mintear
  credenciales, sabiendo que ambas tareas van a chocar contra una credencial faltante, es sembrar un
  disparo de `always_escalate` en el trabajo de un worker. Barato de arreglar ahora, feo después.
- **Rechazar el grafo por el vault** — rechazada: convierte un candado de credencial en un defecto de
  diseño y quema la ventana entera por algo que solo el operador destraba.
- **Escalar por el vault** — rechazada: ningún threshold dispara, y "el operador debe destrabar el
  vault" es un punto de resume, no una pregunta de gobernanza. Se reporta, no se escala.
- **Dejar el comentario upstream como tarea de lane con guardrails fuertes** — rechazada: ninguna
  cantidad de guardrails convierte un acto público irreversible en algo apropiado para un worker
  autónomo. La delegación no da para eso.
- **Diferir la evidencia viva marcándola "deferred" dentro de T-001/T-002** — rechazada: dejaría dos
  tareas en estado ambiguo (parcialmente aceptadas) y sin nodo que represente el gate del operador.
  T-003 lo hace explícito, con `blockedBy` real y precondición nombrada.

## What would flip this

- **Si el operador destraba el vault durante el ciclo:** T-003 deja de estar gated, corre en línea, y
  el ciclo sí cierra bajo su propio criterio. Nada más cambia.
- **Si /cto rechaza AM-1 y mantiene `block/buzz` en el grafo:** entonces `task_graph_repos_gt: 1`
  **sí dispara** y el refute-pass sobre el approve de Phase 5 pasa a ser obligatorio. Las dos cosas
  van juntas; no se puede quedar el segundo repo sin el refute.
- **Si al portar aparece que la semántica `max(head+1, now)` no satisface alguna de las dos
  propiedades** en el fixture real: deja de ser port y vuelve a ser decisión de diseño → nuevo gate.
- **Si T-001 revela que el helper necesita cambios** para funcionar desde el release: choca con el
  guardrail "no modificar `crates/git-credential-nostr/`" (`out_of_scope` del brief) → escala, no se
  resuelve en lane.

## Blast radius (F1c)

Evaluación de `blast_radius_thresholds.triggers` para el approve de Phase 5 **una vez aplicadas las
enmiendas**:

| Trigger | ¿Dispara? | Por qué |
|---|---|---|
| `task_graph_repos_gt: 1` | **No, con AM-1** | El grafo pasa a tocar un repo (`cofoundy/buzz`). El comentario upstream sale a acción de CTO. |
| `files_touched_gt: 25` | No | T-001 toca 3 archivos, T-002 uno, más dos reports. ~6. |
| `deploy_any` | No | Superficie de Phase 11. |
| `architecture_external_surface` | No | Es trigger de cambio arquitectónico; ya tomó su refute en Phase 2 (SUSTAIN). |

**Lo declaro con su condicional porque me conviene el resultado y eso es justamente cuando hay que
mirarlo dos veces.** AM-1 tiene el efecto lateral de bajar el grafo por debajo del umbral. AM-1 se
sostiene **por sus méritos** — un acto público irreversible no es tarea de worker (cuatro razones
arriba) — y yo mismo levanté este umbral en Phase 2 cuando me costaba. Pero el condicional queda
escrito: **si /cto no aplica AM-1, el umbral dispara y el refute es obligatorio.** Y si /cto prefiere
correr el refute igual sobre este approve, no me opongo — `below_threshold: no_refute` es una
optimización de costo, no un derecho del gate.

## Sources

- `.cofoundy/tasks/T-001.md`, `.cofoundy/tasks/T-002.md`, `.cofoundy/specs/file-ownership-matrix.md`,
  `.cofoundy/specs/architecture-v1.md` (enmendado) — artefactos bajo gate.
- `.cofoundy/brief.yaml` — `mvp_scope` (los 4 ítems, cruzados uno por uno en §(d)), `out_of_scope`,
  `Criterio de cierre`.
- `.cofoundy/context/constraints/escalation-thresholds.yaml` — **`required_keys` re-asertados en esta
  pasada**: los 6 presentes y parseables. Verifiqué además con `git log` que el archivo **no fue
  tocado** por el commit de substrate `1722507f` (último cambio: `c0fe675b6`, el scaffold) — la
  config bajo la que decido es la misma que leí en Phase 2. Evaluación: `scope_expansion_request`
  aplica a Phase 5 y **no dispara** (ninguna tarea excede `mvp_scope`, ningún `scope.write` invade
  celda ajena); `budget_overrun` no aplica (sin presupuesto declarado); `production_main_deploy` y
  `security_finding_high` son de Phases 11/8; `always_escalate.credential_mint_or_rotate` **no
  dispara — y AM-3 es lo que lo mantiene así.**
- Refute-pass F1c sobre Phase 2: SUSTAIN con tres premisas refutadas, bajadas a substrate en
  `1722507f`. Las tres incorporadas a este análisis; la del CI upstream cambia lo que T-002 puede
  heredar y ya está reflejada en el propio T-002 (§"El CI upstream no es garantía de nada acá").
- Código verificado: `repos.rs:523` y `:717` son tests in-file (`#[cfg(test)]`), lo que confirma que
  el `scope.write` de T-002 no necesita ampliarse.
- Prior decisions: `2026-08-04-phase-2-architecture.md` (approved, SUSTAIN). **Sin contradicción** —
  esta decisión ejecuta sus IC-1/2/3, no las revisa. AM-1 formaliza IC-3 en el grafo; no la cambia.

## Next action

1. /cto aplica **AM-1** (matriz: un repo + puntero a IC-3), **AM-2** (crear `T-003`, mover la
   aceptación 4 de T-002), **AM-3** (guardrail anti-minteo en T-001 **y** T-002), **AM-4** (línea de
   push end-to-end en T-003).
2. Re-gate Phase 5 — espero `approve` directo si las cuatro aterrizan.
3. Ese approve queda **bajo umbral** de blast radius (con AM-1 aplicada) → sin refute obligatorio.
   Si /cto conserva el segundo repo, **el refute vuelve a ser obligatorio**.
4. Dispatch: **T-001 ∥ T-002**, en paralelo, sin rama de integración (matriz §Validación: superficie
   compartida cero). Ambas 100% desbloqueadas.
5. `T-003` queda `blocked` esperando al operador. **No se despacha, no se simula, no se saltea.**
6. Acción de CTO upstream (comentario en #2876 bajo IC-3) **después** de T-003, nunca antes — sin la
   reproducción viva no tenemos el aporte que justifica el comentario.

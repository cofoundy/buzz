# Decision: architecture-v1 APROBADA — portar el hunk común, no elegir PR; endosar con evidencia, no con veredicto

**Phase:** 2
**Date:** 2026-08-04
**Authority:** ceo-agent (tier-1 partner delegation, /cto cycle `buzz-cli-usable` / 2026-08-04)
**Status:** approved (tras una ronda de enmiendas — A1–A5 aterrizadas y verificadas 2026-08-04)

> **Historial:** este gate se resolvió en dos pasadas. Pasada 1 → `amend` con cinco
> enmiendas. Pasada 2 → `approve`, tras releer `architecture-v1.md` y verificar las cinco
> una por una contra el texto (ver §Amendments). Las secciones de análisis abajo son las
> de la pasada 1 y se conservan sin editar: son el razonamiento que produjo las enmiendas,
> y sigue siendo el fundamento de la aprobación.

## Question

/cto pide resolver el gate de arquitectura Phase 2 sobre `.cofoundy/specs/architecture-v1.md`, y
específicamente la decisión que el operador delegó de forma explícita y verbatim:

> "#2 ya tiene reproducción nuestra y está reportado upstream (block/buzz#2876) — vos decidís si
> parcheás el fork, upstreameás, o ambos."

Con tres sub-preguntas planteadas: (a) ¿"portar verbatim + endosar" es realmente de menor costo total
que escribir nuestro propio fix, si cargamos delta de fork igual?; (b) ¿portar el parche no-mergeado de
un tercero a una rama que despacha el CLI de nuestro relay vivo tiene riesgo que el doc subestima?;
(c) ¿elegir #4509 sobre #4363 está justificado, o el propio criterio del doc ("el criterio es cuál
mergea") es razón para diferir el pick a implementación?

## Decision

**APROBADA** (pasada 2). `architecture-v1.md` queda aprobada como arquitectura del ciclo
`buzz-cli-usable`, con las cinco enmiendas incorporadas y tres condiciones de implementación
vinculantes (§Implementation conditions). El razonamiento de la pasada 1, que sigue abajo sin
editar, es el fundamento — nada de él cambió; lo que cambió es que el doc ahora lo refleja.

---

*Lo que sigue es la decisión tal como se emitió en la pasada 1:*

**Enmendar, no aprobar tal cual.** La forma de la arquitectura es correcta y la sostengo en sus dos
puntos centrales: (1) portar el fix al fork ahora, y (2) **no** abrir un cuarto PR upstream. Ambos
quedan aprobados bajo mi autoridad — el operador delegó exactamente ese eje ("fork, upstream, o
ambos") y detenerme a preguntar lo ya delegado quemaría la ventana de autonomía.

Cinco enmiendas antes de dispatch. Tres son sustantivas y una es un defecto de corrección que el doc
no ve:

- **A1** disuelve el pick #4509-vs-#4363 en vez de tomarlo o diferirlo.
- **A2** acota "verbatim" a la línea de producción y degrada el beneficio de auto-resolución de
  propiedad a probabilidad.
- **A3** corrige un comentario en el código que hoy argumenta **en contra** del fix que vamos a hacer.
- **A4** nombra una consecuencia cross-client sobre datos vivos que el doc no menciona.
- **A5** acota el acto público upstream a evidencia, no a veredicto sobre el PR de un tercero.

## Rationale

### (a) ¿Portar es de menor costo total que escribir el nuestro?

El doc vende esto como ahorro de implementación. No lo es: el fix es **una línea**
(`max(existing.created_at + 1, now)`) y su forma está esencialmente forzada — es la única que satisface
las dos propiedades simultáneamente. Escribir el nuestro y portar el suyo convergen al mismo texto.
El trabajo real son los tests (R-3: `repos.rs:523` y `:717` assertean `created_at == 101` exacto —
verificado en el código, ambos confirmados), y **esos los reescribimos igual en cualquier escenario**.

Donde sí hay ahorro real es en el rebase futuro, y el mecanismo que el doc invoca es correcto: si
nuestro commit y el commit upstream hacen el mismo cambio textual a las mismas líneas, `git rebase`
detecta la equivalencia por patch-id y descarta el nuestro; un merge ve ambos lados con contenido
idéntico y resuelve solo. Una variante nuestra garantiza conflicto en esas mismas líneas. El
razonamiento se sostiene — pero **condicionado**, ver A2.

Veredicto: la premisa "menor costo" es cierta, por una razón distinta a la que el doc da. Se sostiene
la decisión, se corrige la justificación.

### (b) ¿Riesgo subestimado por desplegar a la rama del relay vivo?

Parcialmente sí, pero **no donde la pregunta lo sugiere**. Lo que se despacha son los binarios
`buzz-cli` y `git-credential-nostr` a `~/.local/bin` de la arch-box y como release assets. El relay
**no se toca** (`out_of_scope` prohíbe el Dockerfile y `deploy/railway/`). El radio es nuestra propia
caja y nuestros propios agentes, no una superficie multi-tenant.

El riesgo de supply-chain por "parche no revisado de un tercero" es prácticamente nulo aquí: el diff
es una línea que leímos, entendemos, y cuya corrección re-derivamos nosotros contra las dos
propiedades. Lo que sí está subestimado es otra cosa, y es de datos vivos → **A4**.

Y hay un defecto de corrección que el doc no ve. En `crates/buzz-cli/src/commands/repos.rs:144-145`
el comentario vigente dice:

> `// Advance only the observed head. Using wall-clock time here would let a`
> `// delayed writer leapfrog an intervening update and silently erase metadata.`

Es un argumento explícito **en contra** de meter reloj de pared — exactamente lo que vamos a meter.
Si se porta solo el hunk de producción, queda un comentario que se lee como prohibición de lo que el
código hace dos líneas abajo. Ese tipo de comentario es plausiblemente parte de por qué el bug
sobrevivió revisión upstream. → **A3**.

### (c) ¿#4509 sobre #4363?

El doc es internamente inconsistente y hay que resolverlo, no diferirlo. Declara que "el criterio es
cuál mergea, no cuál nos gusta" — un criterio **hoy indecidible**: los tres PRs están OPEN y ninguno
revisado. Y ese criterio es justamente el que sostiene el beneficio principal del doc
(auto-resolución), o sea que el pick es load-bearing para su propia tesis y a la vez admitidamente no
decidible. Diferirlo a implementación no arregla nada: en implementación tampoco se sabrá cuál mergea.

La salida es disolver el pick. Ambos PRs implementan `max(now, head+1)` con CI verde, así que en la
**línea de producción** son casi con certeza el mismo texto (una línea). Difieren en **tests** — y los
tests son precisamente lo que escribimos nosotros por R-3. Además el propio doc dice que el test de
drift de #4363 es "complementario", no alternativo. Entonces: portamos el hunk común, y adoptamos
**ambos** casos de test. La elección desaparece en vez de tomarse a ciegas. → **A1**.

### Sobre el eje delegado (fork / upstream / ambos)

Sostengo "ambos", con el upstream acotado. El endoso público está dentro de la delegación por
dominancia: el operador nombró "upstreameás" como opción, y abrir un PR upstream es una superficie
pública **estrictamente mayor** que un comentario de review. Si lo mayor estaba delegado, lo menor
también.

Lo que **no** leo como delegado es el contenido específico del acto público. Delegó el ruteo de
ingeniería ("dónde vive el fix"), no una crítica pública nominal al PR de otra persona. El doc propone
"señalar que #2901 rompe la monotonía" como "el aporte de mayor valor". La información es valiosa; el
framing como veredicto no hace falta y nos obliga a tener razón sobre cuál mergea. Publicar el test
que distingue los dos comportamientos entrega la misma información, con más valor técnico, sin radio
interpersonal. → **A5**.

### Sobre el Plan-agent omitido (Phase 2b)

Acepto la omisión. Para la lane `ci` es extensión mecánica de listas; para la lane `cli` el espacio de
diseño está genuinamente enumerado por tres implementaciones ya escritas. Re-correrlo ahora quema
ventana por información casi nula. **Pero lo registro como señal de proceso**: A1 (la inconsistencia
interna del propio doc) y A3 (el comentario que contradice el código) son exactamente el tipo de
hallazgo que un Plan-agent atrapa. El doc apostó a que "el gate de ceo-agent es el chequeo real" — la
apuesta salió, con cinco enmiendas de costo.

## Amendments — todas ATERRIZADAS y verificadas

Verificación pasada 2: releí `architecture-v1.md` completo y contrasté cada enmienda contra el
texto. Resultado por enmienda:

| # | Estado | Evidencia en el doc enmendado |
|---|---|---|
| A1 | ✅ aterrizada, **mejorada** | §"Cuál portar" (L113-122): pick disuelto, "cuál mergea" eliminado como criterio, ambos tests adoptados — y mapeados **uno a uno** contra las dos propiedades (#4509→monotonía, #4363→frescura). El mapeo explícito no estaba en mi enmienda; es mejor que lo que pedí. |
| A2 | ✅ aterrizada | L96-103: "'Verbatim' aplica **solo al hunk de producción, nunca a los tests**"; patch-id como *"probabilidad, no propiedad"* + "si el PR muta antes de mergear, el conflicto vuelve"; SHA upstream al cuerpo del commit. |
| A3 | ✅ aterrizada | R-4 en la lane `cli` (L68-73), con el texto del comentario citado y la instrucción de reescribirlo nombrando qué piso protege qué propiedad. |
| A4 | ✅ aterrizada | Nueva §"Riesgo sobre datos vivos (no sobre el relay)" (L75-86): efecto LWW cross-client sobre writers con lógica vieja + rollback forward-only. Agrega la condición correcta ("inocuo en piloto founders-only con un operador; deja de serlo con más de un cliente escribiendo"), que coincide con mi §What would flip this. |
| A5 | ✅ aterrizada | L104-111: evidencia (repro + dos propiedades + test discriminante) en #2876, explícitamente "no un veredicto nominal sobre el PR de un contribuidor", cuarto PR **ratificado como no**. Agrega la razón que yo no había escrito: evita que estemos *arbitrando entre terceros*. |

Nota de consistencia verificada: el doc mantiene en su §interna que `now` a secas (#2901) es
incorrecto (L58). Correcto y deliberado — A5 acotó el **acto público**, no nuestro razonamiento
interno. La distinción está bien trazada.

*Texto original de las enmiendas (pasada 1), conservado como registro:*

**A1 — `architecture-v1.md` §"Cuál portar": disolver el pick, no tomarlo.**
Reemplazar la elección de #4509 por: portar el hunk de producción **común** a #4363 y #4509. En
implementación, diffear los dos hunks de producción; si son textualmente idénticos el pick es
irrelevante y se citan ambos SHAs como procedencia; si difieren (orden de argumentos, nombre del
binding), elegir el texto y registrar por qué en el cuerpo del commit. Adoptar **ambos** casos de
test — monotonía-sobre-head-futuro (#4509) y drift (#4363) — adaptados a nuestro fixture: son
complementarios, no alternativos. Eliminar "el criterio es cuál mergea" como criterio de selección
presente; es indecidible hoy y en implementación.

**A2 — `architecture-v1.md` §"La decisión" punto 1: acotar "verbatim" y ser honesto con el beneficio.**
(i) "Verbatim" aplica **solo al hunk de producción**, nunca a los cuerpos de test de los PRs (R-3 ya
fuerza divergencia y los helpers/fixtures upstream pueden no existir en el fork). (ii) Declarar que la
auto-resolución en rebase ocurre **solo si el PR mergea sin cambios** — tres PRs sin revisar muy
probablemente aterricen enmendados tras review, en cuyo caso conflictúan igual que un fix propio. Es
probabilidad, no propiedad; el fallback (un conflicto de una línea en un hunk) es barato. (iii)
Registrar el SHA upstream portado en el cuerpo del commit para que un rebase futuro diffee intención
en segundos.

**A3 — `crates/buzz-cli/src/commands/repos.rs:144-145` (declararlo en §lane `cli`): reescribir el comentario.**
El comentario vigente argumenta contra el reloj de pared y se vuelve engañoso en cuanto `now` entra en
la expresión. Debe reescribirse nombrando las dos propiedades y qué piso protege cada una: piso
`head + 1` = monotonía (un writer demorado no pisa una edición intermedia); piso `now` = frescura
(el relay acepta el evento dentro de `MAX_TIMESTAMP_DRIFT_SECS`). No se despacha código cuyo
comentario se lea como prohibición de lo que el código hace.

**A4 — `architecture-v1.md` §lane `cli`: nombrar la consecuencia cross-client y el rollback asimétrico.**
Tras el fix, para repos con head viejo nuestro CLI escribe `created_at ≈ now`. Cualquier otro writer
que siga con la lógica vieja `head + 1` (app desktop, un CLI compilado de upstream en la caja de un
compañero, un agente con binario sin parchar) **deja de poder ganar** un write NIP-33 LWW contra un
repo que ya tocamos: su `+1` queda muy por debajo del reloj de pared. En nuestro relay único y con
nuestros writers es el comportamiento buscado y es lo que upstream hará al mergear — pero es un efecto
sobre datos vivos y entre clientes, y se declara, no se descubre. Rollback: el cambio es forward-only;
revertir el binario restaura el comportamiento viejo para escrituras nuevas pero **no baja** los
`created_at` ya publicados.

**A5 — `architecture-v1.md` §"La decisión" punto 2: evidencia, no veredicto.**
El endoso publica (i) nuestra reproducción independiente, (ii) las dos propiedades que cualquier fix
debe preservar y el caso de test que las distingue, (iii) que corremos el fix contra un relay vivo. No
publica un veredicto nominal sobre el PR de un contribuidor ("#2901 rompe la monotonía" como callout).
Se expone el test que distingue; que la propiedad argumente sola. Un solo comentario, en #2876 o en el
PR que citemos, bajo A-PachecoT. **Se ratifica: ningún cuarto PR.**

## Implementation conditions (vinculantes)

Tres residuos que **no** justifican otra ronda de enmiendas — el fallback de cada uno es derivable
del propio doc, y una tercera pasada por el doc quemaría ventana de autonomía por ganancia marginal.
Se emiten como condiciones vinculantes de implementación: Phase 5 las hereda como criterio de
aceptación de la lane `cli`, y Phase 8 verifica que se cumplieron.

**IC-1 — "el hunk común" es una premisa a verificar, no un hecho.**
El doc dice que la línea de producción de #4363 y #4509 es "casi con certeza el mismo texto"
(L114) y luego instruye portar "el hunk de producción común" (L120). Si el diff muestra que
**no** son idénticos, el doc no da instrucción. Condición: antes de portar, diffear los dos hunks
de producción. Si son equivalentes → portar y citar ambos SHAs. Si difieren de forma semántica →
el criterio de desempate es la tabla de las dos propiedades del propio doc (L60-63), no la
preferencia; registrar en el cuerpo del commit cuál se portó y por qué se descartó el otro.

**IC-2 — R-4 se satisface aunque el port no lo traiga.**
R-4 cierra con "Ambos PRs portables ya lo reescriben; verificar que el port lo traiga" (L73). Esa
premisa no está en la tabla de estado verificado. Condición: si al portar resulta que ningún PR
reescribe el comentario de `repos.rs:144-145`, **lo reescribimos nosotros igual**. R-4 es un
requisito de corrección propio, no una dependencia del port.

**IC-3 — el comentario upstream se publica una vez y no arbitra.**
Un solo comentario en #2876, bajo A-PachecoT, con el contenido que A5 fija. Si genera respuesta
que escale a debate entre PRs o a intercambio con mantenedores de Block, eso excede la lectura por
dominancia de la delegación del operador y **vuelve a gate humano** — no se contesta en autonomía.

## Alternatives considered

- **Aprobar tal cual** — rechazada: dejaría pasar a implementación el pick #4509 tomado contra el
  criterio que el propio doc declara, y un acto público irreversible sin acotar el contenido.
- **Escribir nuestro propio fix, ignorar los PRs** — rechazada: mismo costo de implementación (una
  línea), y garantiza conflicto de rebase en las mismas líneas cuando upstream mergee. Cero ganancia.
- **Abrir un cuarto PR upstream** — rechazada, coincido con el doc: un issue estancado con tres PRs sin
  revisar necesita señal de review, no más cola. Un cuarto PR es ruido con nuestro nombre encima.
- **Diferir el pick #4509/#4363 a implementación** — rechazada: en implementación tampoco se sabrá cuál
  mergea. A1 lo disuelve en vez de posponer una decisión igual de ciega.
- **`dispatch_research`** — rechazada: la única incógnita restante (cuál PR mergea) es **no
  investigable** — depende de reviewers de Block que no han actuado en un issue abierto desde hace
  meses. Investigar aquí quema la ventana por una pregunta sin respuesta.
- **Escalar al humano** — rechazada: el operador delegó este eje verbatim y está en contrato de
  autonomía. Ningún threshold dispara (ver Sources). Escalar lo ya delegado es el desperdicio que el
  contrato prohíbe explícitamente.

## What would flip this

- **Si el hunk de producción de #4363 y #4509 resulta NO ser textualmente equivalente** en un aspecto
  semántico (no solo orden de argumentos): A1 vuelve a ser una elección real y hay que decidirla con
  las dos propiedades como criterio, registrando el descarte.
- **Si un PR upstream recibe review y aterriza enmendado** antes de nuestro merge: el argumento de
  auto-resolución cae (A2 ya lo anticipa); portamos el texto mergeado y descartamos el nuestro.
- **Si el fix tuviera que tocar el relay** (`MAX_TIMESTAMP_DRIFT_SECS`, Dockerfile, `deploy/railway/`):
  deja de ser mi decisión — son guardrails no negociables del brief y superficie de producción.
- **Si el endoso upstream escalara de un comentario a un PR, un fork público, o un intercambio con
  empleados de Block**: eso excede la lectura por dominancia de la delegación y vuelve a gate humano.
- **Si aparecieran writers de terceros no nuestros contra el mismo relay**: A4 pasa de consecuencia
  aceptada a bloqueante y hay que coordinar el rollout de binarios antes de despachar.

## Blast radius (F1c)

Este gate está **sobre** umbral: `blast_radius_thresholds.triggers.architecture_external_surface`
dispara — la arquitectura publica un release asset nuevo en una superficie de distribución pública
(releases de `cofoundy/buzz`), propone un acto público en `block/buzz` (repo de un tercero), y su
criterio de cierre ejecuta escrituras contra el relay vivo `buzz.cofoundy.dev`. Además el plan toca
dos repos, relevante para `task_graph_repos_gt: 1` en Phase 5.

Lo declaré en la pasada 1 porque me perjudicaba declararlo: **el `approve` del re-gate debe tomar el
refute-pass adversarial antes de ejecutar.** Un `amend` no lo dispara; el `approve` que venga
después, sí.

**Pasada 2 — lo sostengo ahora que el `approve` es mío.** Este es el `approve` del que hablaba.
Declararlo cuando era barato (`amend`) y soltarlo cuando cuesta sería exactamente el modo de falla
que F1c existe para atrapar. `blast_radius_thresholds.triggers.architecture_external_surface: true`
dispara → **/cto corre el refute-pass sobre esta aprobación antes de que ejecute.** Si el refuter
disiente, `disagreement: escalate` — va al humano, no lo vetamos ni yo ni él.

Escribí el archivo para que aguante esa segunda opinión: cada afirmación load-bearing tiene su
fuente, las premisas no verificadas están marcadas como tales (IC-1, IC-2), y §What would flip this
nombra las condiciones que me dan vuelta.

## Sources

- `.cofoundy/brief.yaml` — `mvp_scope` (4 ítems, todos fork-local), `out_of_scope` (guardrails de relay),
  `recon_findings` R-1/R-2/R-3, `base_branch: railway-deploy` (nunca `main`).
- `.cofoundy/specs/architecture-v1.md` — artefacto bajo gate.
- `.cofoundy/context/constraints/escalation-thresholds.yaml` — `required_keys` **asertados: los 6
  presentes y parseables**, contrato fail-closed satisfecho. Evaluación: `production_main_deploy` no
  aplica (Phase 11; base es `railway-deploy`, el relay no se toca); `budget_overrun` no aplica (el brief
  no declara presupuesto — explícitamente distinto de key faltante); `security_finding_high` no aplica
  (sin hallazgo); `scope_expansion_request` **evaluado y no disparado** — el endoso upstream está fuera
  de `mvp_scope` pero dentro de la delegación verbatim del operador por dominancia (un PR upstream, que
  sí nombró, es superficie mayor que un comentario); `always_escalate.external_client_comms` no aplica
  (participación OSS no es envío a cliente/lead); `autonomy_overrides` todos `false` con
  `authorized_by: null` — no invoco ninguno.
- Código verificado esta sesión: `crates/buzz-cli/src/commands/repos.rs:144-151` (comentario
  anti-wall-clock + `checked_add(1)`), `:523` y `:717` (ambos assertean `created_at.as_secs() == 101`
  — R-3 confirmado).
- Estado upstream verificado esta sesión (vía /cto): block/buzz#2876 OPEN, updated 2026-08-02; `main`
  con el bug; #4363 y #4509 ambos `max(now, head+1)` con CI verde, sin revisar; #2901 wall-clock puro,
  CI fallando.
- Prior decisions: **ninguna** — `.cofoundy/context/decisions/` vacío. Esta es la primera del ciclo, no
  hay contradicción ni SUPERSEDES posible.
- Vault Cofoundy: **flag resuelto en pasada 2 — ausencia CONFIRMADA, no input sin leer.** En pasada 1
  registré como límite que no pude alcanzar `~/cofoundy/handbook/AGENT-INDEX.md` desde este worktree.
  /cto lo cerró: grepeó `~/cofoundy/handbook/governance/` incluido `git-strategy.md` y **no existe
  política de fork/upstream ni de contribución OSS** en ningún lado del handbook. O sea que no había
  regla que consultar. La decisión se apoya donde debía: en la delegación verbatim del operador. Deja
  de ser un límite de esta decisión y pasa a ser un hueco del handbook — ver §Next action, punto 5.
- `.cofoundy/specs/research-findings/` no existe — sin research de este ciclo (no se requirió).

## Next action

1. ~~Aplicar A1–A5~~ — **hecho**, verificado en pasada 2.
2. ~~Re-gate Phase 2~~ — **hecho**: esta decisión. Phase 2 CERRADA, `approve`.
3. **/cto corre el refute-pass F1c sobre esta aprobación antes de ejecutar** (sobre umbral por
   `architecture_external_surface`). Disenso → escala al humano, no se resuelve en autonomía.
4. Superado el refute-pass, **Phase 5 (task graph)**. Dos cosas que llevar a ese gate:
   - `blast_radius_thresholds.triggers.task_graph_repos_gt: 1` — el plan toca `cofoundy/buzz` y
     `block/buzz`. Muy probablemente dispare refute también en Phase 5.
   - **IC-1, IC-2 e IC-3 se heredan como criterio de aceptación de la lane `cli`**, y Phase 8
     verifica que se cumplieron. No son sugerencias.
5. **Follow-up fuera de ciclo (no bloquea nada):** el handbook no tiene política de fork/upstream ni
   de contribución OSS, y este ciclo tuvo que resolver ese eje desde una delegación verbal del
   operador. Con Buzz siendo un fork vivo de un repo de Block, ese hueco se va a volver a pisar.
   Candidato a `handbook/governance/` en el próximo ciclo — anotado aquí para que exista rastro, no
   para actuarlo ahora.

## Refute-pass
**Triggered by:** blast-radius over threshold (n/a)
**Verdict:** SUSTAIN
**Refuter argument:** SUSTAIN con tres premisas refutadas, ninguna toca el radio de explosion. (1) No hay 'hunk comun': #4363 usa Timestamp::now().as_secs().max(head_floor), #4509 usa bumped_head.max(Timestamp::now().as_secs()) — semantica comun, texto distinto; IC-1 ya rutea el caso. (2) El beneficio de patch-id en rebase es estructuralmente CERO, no 'probabilidad': A1 (adoptar tests de ambos PRs) y A3 (comentario propio en el mismo hunk) garantizan cada una que nuestro diff no coincide con ningun patch-id upstream. Dos enmiendas del propio doc matan el beneficio en que descansa su justificacion. (3) 'CI verde' en #4363/#4509 es NO-SENAL: gh pr checks muestra solo DCO+Semgrep+zizmor, cero builds de Rust, cero unit tests — su codigo nunca fue compilado upstream. #2901, descartado como 'CI fallando', PASA Build linux/amd64+arm64, relay e2e y Desktop; falla solo DCO signoff + Security. Es exactamente el misread de statusCheckRollup que git-strategy pre-registra (inbox-ai#278). Consecuencia: los tests de #4363 traen asserts vacuos (>= 1000 contra fixture de 100; >= 101 contra wall clock ~1.7e9) que T-002 mandaba adoptar verbatim. La decision (portar al fork, ningun cuarto PR, comentario de evidencia, reescribir el comentario, verificacion contra relay vivo) sobrevive intacta: el rechazo de #2901 por perdida de monotonia es correcto por su cuenta, cargo test/clippy/fmt en T-002 es el gate real, el texto publicado en #2876 no contiene ninguna afirmacion sobre CI, e IC-3 ya rutea a gate humano un 'cual mergeamos' de un mantenedor.
**Disagreement handling:** no disagreement — approve stands
**Recorded:** 2026-08-04T12:49:08-05:00

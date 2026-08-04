# file-ownership-matrix — buzz-cli-usable

Roles = columnas. `W` = escribe · `R` = lee · `A` = append-only · vacío = sin acceso.

| Path glob                                          | ci | cli | notas |
|----------------------------------------------------|----|-----|-------|
| `.github/workflows/buzz-acp-linux.yml`             | W  |     | issue #1 — matriz de build + assets |
| `deploy/arch-box/install-buzz-acp.sh`              | W  |     | issue #1 — instalación del helper |
| `deploy/arch-box/README.md`                        | W  |     | issue #1 — **no existe**, lo crea (R-1) |
| `crates/buzz-cli/src/commands/repos.rs`            |    | W   | issue #2 — el fix + sus tests |
| `crates/git-credential-nostr/**`                   | R  |     | guardrail: NO modificar |
| `crates/buzz-relay/src/handlers/ingest.rs`         |    | R   | guardrail: NO modificar (MAX_TIMESTAMP_DRIFT_SECS) |
| `deploy/railway/**`, `Dockerfile*`                 | R  | R   | guardrail explícito: fuera de scope |
| `.cofoundy/state/reports/<role>.md`                | W  | W   | cada lane escribe SOLO su propio archivo |
| `.cofoundy/state/history.jsonl`                    | A  | A   | append-only |
| `.cofoundy/**` (resto)                             | R  | R   | el CTO es el único writer |

## Validación

Ninguna celda tiene 2+ `W`. Las dos lanes son **disjuntas por construcción**: `ci`
vive enteramente en `.github/` + `deploy/arch-box/`, `cli` en un solo archivo Rust.
Cero superficie compartida ⇒ ambas pueden correr en paralelo y aterrizar por su
cuenta (Phase 8 per-lane autonomous landing), sin rama de integración.

`.cofoundy/state/reports/<role>.md` es un archivo por rol — nombre único, colisión
de merge imposible. El índice compartido lo escribe solo el CTO.

## Repos tocados

**Uno: `cofoundy/buzz`.** (Corregido en el gate de Phase 5, AM-1.)

El comentario de evidencia en `block/buzz#2876` **no está en el grafo de tareas** — es
una **acción de CTO**, fuera de las lanes, por cuatro razones:

1. No tiene aceptación testeable: es un acto comunicativo, no compila ni corre.
2. Es irreversible y público bajo identidad de la organización, en el repo de un
   tercero — asimétrico respecto de todo lo demás del grafo, revertible con `git revert`.
3. El operador delegó esa decisión **al gate**, no a un worker. Una lane publicando en
   `block/buzz` sería una delegación estrictamente mayor que la que se hizo.
4. Su contenido es gobernanza (evidencia, no veredicto; no arbitrar entre terceros),
   no implementación. Los workers cargan specs, no gobernanza.

Restricción vinculante: **IC-3 — un solo comentario.** Si genera debate entre PRs o
intercambio con mantenedores de Block, excede la delegación y vuelve a gate humano.

Secuencia: `T-001 ∥ T-002 → T-003 → acción CTO upstream`. El comentario está bloqueado
por el mismo candado de vault que T-003: su aporte diferencial es la reproducción
contra un relay vivo, así que publicarlo antes sería publicar sin la evidencia que lo
justifica.

Ninguna lane escribe en `block/buzz`. Ninguna lane tiene esa ruta en su `scope.write`.

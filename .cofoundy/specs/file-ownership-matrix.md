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

Dos: `cofoundy/buzz` (código) y `block/buzz` (un comentario de evidencia en #2876,
sin PR). Se declara acá porque `task_graph_repos_gt: 1` es un umbral que el gate de
Phase 5 evalúa — el segundo repo es una acción de solo-comentario, no un cambio de
código.

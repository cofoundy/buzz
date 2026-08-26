# Kickoff listo para la próxima sesión — `buzz-cli-usable`

Copiá esto tal cual al arrancar. Re-fundamentá en el substrate (Step 2 de `/cto`
infiere la fase del disco) y parate en los gates.

---

Sos el sitio `buzz-cli-usable`. Rama `sprint/buzz-cli-usable` @ `b7f5b2c38`, base
`railway-deploy`, **nunca `main`**. Estado y decisiones del ciclo anterior en
`.cofoundy/` — leelo antes de actuar, no re-derives.

**Lo hecho (2026-08-04):** T-001 y T-002 mergeados y pusheados. cofoundy/buzz#1 y #2
arreglados y verificados: 273 tests, clippy y fmt limpios, release
`buzz-acp-linux-f40de5a` con los tres binarios verificado por checksum. El fix de #2
se **portó** de block/buzz#4363/#4509 en vez de escribirse — ya había tres PRs
abiertos upstream.

**Lo pendiente, en orden:**

1. **T-003 — la evidencia viva.** Es lo único que separa este ciclo de estar cerrado;
   el criterio del brief es evidencia contra `buzz.cofoundy.dev`, no build verde.
   Precondición: Andre corre `bw unlock` y exporta `BUZZ_PRIVATE_KEY`. Spec completa
   en `.cofoundy/tasks/T-003.md`. Tres pruebas: `repos protect set` sobre un repo de
   >15 min, el helper en `~/.local/bin` de una caja sin toolchain, y `clone`+`push`
   siguiendo **solo** `deploy/arch-box/README.md`.
2. **El comentario a block/buzz#2876.** Redactado en
   `.cofoundy/state/upstream-2876-comment.draft.md`, sin publicar. Va **detrás** de
   T-003 porque su aporte diferencial es la reproducción contra relay vivo. Gobernado
   por IC-3: **un solo comentario**; si genera debate entre PRs o intercambio con
   mantenedores de Block, vuelve a gate humano.
3. **Publicar el exec-summary en Basalt** (`docs/reports/buzz-cli-usable.md`). Requiere
   OAuth interactivo — `mcp__basalt__authenticate` y que Andre autorice en el browser.
4. Cerrar cofoundy/buzz#1 y #2 recién cuando corra T-003, no con el merge.

**Contrato de autonomía que sigue vigente:** gates vía `ceo-agent`, nunca
`AskUserQuestion`; escalaciones a `.cofoundy/state/escalation-queue.jsonl`; y la
prohibición que el gate de Phase 5 hizo textual — **no generar, mintear ni rotar
credenciales bajo ninguna circunstancia**. Si falta una, la tarea se detiene y lo
reporta.

**Decisiones que son de Andre, no tuyas:**

- Autorizar el comentario upstream (acto público e irreversible en el repo de Block
  bajo su identidad).
- Si `sprint/buzz-cli-usable` va a `railway-deploy` por PR o directo, y cuándo.
- Si el handbook necesita una política de fork/upstream. Este ciclo resolvió ese eje
  desde una delegación verbal; con Buzz siendo un fork vivo de Block, el hueco se
  vuelve a pisar.

**Kill switch:** si el relay está caído o `bw` sigue locked, no hay nada que hacer acá
— reportá y parate. No inventes evidencia, no generes llaves.

# Thin maintenance fork

This branch maintains a small, reviewable patch set on upstream WebCodex v0.4.1.
It is not an upstream release, a replacement architecture, or a business-data store.

## Branch contract

- `main` follows `yyjeqhc/webcodex` without local product changes.
- `rrx-stable` points only to the source of the most recently accepted installed runtime.
- `rrx/integrate-*` branches contain candidate backports and validation work. A green build alone does not authorize moving `rrx-stable` or replacing a running installation.
- Use fast-forward updates and explicit refspecs. Never force-push or publish all local refs.

The initial accepted runtime baseline is `75ae07b16cb4f0c05a1e07f2ac7cad9afbb8c436`.
Its upstream release base is `f080c8f3ea70e37bd9f17fdd0e1b4c3a3aa330f8`.
The reviewed upstream snapshot for this integration is `37098d4624f0000bfab54ad102ecb0df619d7dc6`.

## Patch inventory

| Patch | Why it is retained | Retirement condition |
| --- | --- | --- |
| Python/TypeScript startup summary and schema | Report providers supported by the existing native LSP implementation | Equivalent upstream summary and schema pass the same tests |
| Mixed-language LSP, Windows URI identity, diagnostic freshness | Allow mixed projects and reject stale diagnostics without widening the Runner-wide process cap | Equivalent upstream fixes pass Windows error/clean/error and capacity tests |
| Markdown artifact inference | Preserve Markdown filenames and exact bytes through supported artifact transport | Equivalent upstream artifact implementation passes resource readback |
| Registration-only lookup for LSP and validation | Avoid scanning unrelated Git repositories on each bounded status request | Upstream lookup avoids Git enrichment while preserving registration and root policy checks |
| MCP timeout fixture separation | Test call deadlines independently from cold process startup | Equivalent upstream fixture isolates both budgets |
| Upstream PR #607 stdin isolation | Raw shell and validation children receive EOF, not the Runner's parent-liveness input | Integrated upstream source includes ae52912cd599d329d95d619e2a272e427d60ab56 or equivalent |
| Upstream PR #610 Runner parent-pipe observation | Windows pipe monitoring must not block registration and must notice parent EOF | Integrated upstream source includes the relevant 37098d4624f0000bfab54ad102ecb0df619d7dc6 behavior |
| Windows standard-pipe inheritance fence | Detached descendants must not retain unrelated parent capture pipes after the Runner exits | Equivalent upstream startup isolation passes the live-descendant capture-EOF regression |

The #607 backport maps the upstream JobManager module back to the v0.4.1 `src/main.rs` location and adapts private test fixtures. Its two real-process tests remain ignored by default and are run explicitly during acceptance.
The #610 backport imports the Runner pipe-listener code and original regression, with required Windows API feature declarations. It does not import unrelated Desktop relocation, UI, Goal, or file-transfer refactoring changes.
No production deadline, authentication, authorization, sandbox, or process-lifetime boundary is relaxed to make tests pass.

The additional standard-pipe fence is a local corrective patch, not part of upstream #607 or #610. It clears only the inheritance flag of this process's existing standard pipe handles before any worker or internal detached mode starts; the handles remain open and usable. The isolated Windows regression keeps a descendant alive while proving parent capture EOF, and separately proves explicitly inherited stdout/stderr still work. Deployment acceptance must also retain a detached Job across Runner replacement with real parent and capture pipes.

## Validation and deployment

Run the dedicated Windows acceptance workflow on the exact candidate commit. It uses a clean hosted Windows machine, read-only repository permissions, and no private credentials or runtime state. Local policy-blocked tests remain blocked observations; hosted success does not turn a local denial into a pass.

Before deployment, additionally verify on the target host: cold and warm startup summaries, mixed-language navigation, fresh diagnostics, guarded writes, artifact byte readback, typed command execution, and actual parent-lease close/restart behavior. Preserve the previous installed binaries and configuration, verify that unrelated tasks are idle, and use the existing Desktop lifecycle. Never retry a failed activation without first inspecting its receipt and current process state.

Advance `rrx-stable` only after the running CLI, Server, and Runner report the same clean source commit and online functional acceptance succeeds. Keep deployment receipts and rollback data outside this public repository.

## Publication and trust boundary

Only generic source, tests, and maintenance documentation belong in this public fork. Do not commit business files, host paths, logs, account identifiers, tokens, certificates, runtime databases, or rollback snapshots.
Do not disable Smart App Control, enterprise application control, antivirus, or code-integrity enforcement. Any local signing or allow-rule change requires identification of the actual policy and approval by its owner. No such policy change is part of this patch set.

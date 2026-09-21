# Semantic navigation startup summary

The bounded coding-startup status probe recognizes all four Runner language-server
profiles: Rust, Go, Python, and TypeScript (the Runner also uses the TypeScript
provider for JavaScript navigation).

The existing Rust-before-Go selection is preserved. When neither is detected,
Python takes precedence over TypeScript. This selects one compact summary; it does
not limit which file-specific navigation tools the Runner can dispatch.

Detection is not an installation or availability check. The selected provider's
observed status determines `available`, `recommended`, and `position_encoding`.
A missing/mismatched provider record remains a malformed result. No server process
is launched by the startup status probe.

Project resolution uses the enabled registry record, not a Git-enriched inventory
scan. LSP and validation do not execute Git in unrelated repositories before
handling a request. The selected root is still checked against Runner policy;
disabled/removed registrations are re-read on every request, and a later duplicate
cannot re-enable a disabled first record. The normal inventory retains its Git
metadata; no stale availability cache or wider permissions are introduced.

The two-second shared probe deadline and cancellation behavior are unchanged.
Timeout remains `probe_timeout` with `available=null`, not a positive or negative
availability claim. Direct `lsp_status` can provide a separate current observation.
The legacy `rust_not_detected` reason is retained when no recognized language was
observed, to avoid changing unrelated wire compatibility.

The diagnostic output schema includes the Python/TypeScript language, provider,
and limitation values. No tools, permissions, or response-size bounds are added.

## Windows shell lifecycle acceptance

Use an independently installed, signature-verified PowerShell executable for a
Runner whose child-process cleanup must be enforced by Windows Job Objects. On
the inspected host, Store-packaged PowerShell 7.6.6 left native descendants alive
after the owning job was terminated; changing the alias to the package's actual
path did not repair it. The official standalone 7.6.6 distribution and Windows
PowerShell 5.1 passed the same parent/descendant test. A Runner-local `shell.program`
and `shell.path_prepend` can select the verified standalone distribution without
changing the user's global PATH or uninstalling the Store app.

Do not weaken Windows application-control policy for generated test executables.
A policy denial is an environment-blocked test, not a successful plugin test or
permission to retry the blocked executable through a different route.

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

The two-second shared probe deadline and cancellation behavior are unchanged.
Timeout remains `probe_timeout` with `available=null`, not a positive or negative
availability claim. Direct `lsp_status` can provide a separate current observation.
The legacy `rust_not_detected` reason is retained when no recognized language was
observed, to avoid changing unrelated wire compatibility.

The diagnostic output schema includes the Python/TypeScript language, provider,
and limitation values. No tools, permissions, or response-size bounds are added.

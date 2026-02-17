# Extract `code_mode` module from mcp-server-common into open-source pmcp SDK

## Problem

The `code_mode` module currently lives in `mcp-server-common` (a private crate
inside the `pmcp-run` repo). Downstream open-source projects like
[OpenClaw/mcp-servers](https://github.com/theonlyhennygod/mcp-servers) depend on it via local path:

```toml
mcp-server-common = { path = "../../../Development/mcp/sdk/pmcp-run/built-in/shared/mcp-server-common", features = ["js-runtime"] }
```

This blocks:
- `cargo install` from git or crates.io
- Distribution as prebuilt binaries
- Any third-party project from using the code_mode execution engine

Since `code_mode` is **completely self-contained** (zero dependencies on other
`mcp-server-common` modules like auth, secrets, config, prompts, or resources),
it is a clean candidate for extraction into the open-source `pmcp` crate on
crates.io.

## What code_mode provides

A secure, auditable pipeline for LLM-generated code execution:

1. **Validation pipeline** — Parses and analyzes scripts before execution
2. **JavaScript/TypeScript parsing** — SWC-based AST analysis (no eval, pure Rust)
3. **Plan compilation** — Converts JS code into an `ExecutionPlan` (auditable AST)
4. **Plan execution** — Runs compiled plans against a pluggable `HttpExecutor` trait
5. **GraphQL validation** — Query analysis and risk assessment
6. **Approval tokens** — HMAC-signed tokens linking validated code to execution
7. **Policy enforcement** — Configurable risk levels and optional AVP integration

## Module stats

- **~16,400 lines** across 18 files
- **Zero imports** from other `mcp-server-common` modules (`grep -r "use crate::" | grep -v "code_mode"` returns 0 results)
- **Only external dependency on pmcp:** `pmcp::types::ToolInfo` (for tool definitions)
- **Well-structured feature flags** already in place:

```
code-mode          -> core (hmac, sha2, graphql-parser, chrono)
openapi-code-mode  -> + SWC parser (swc_ecma_parser, swc_ecma_ast, swc_ecma_visit)
js-runtime         -> + full JS execution engine
mcp-code-mode      -> + MCP-specific executor
avp                -> + AWS Verified Permissions (optional)
dynamo-config      -> + DynamoDB config loading (optional)
```

## Key types to expose

```rust
// Core validation
pub use CodeModeConfig, ValidationPipeline, ValidationContext;
pub use RiskLevel, CodeType, ValidationResult, SecurityAnalysis;

// Token management
pub use ApprovalToken, HmacTokenGenerator, TokenGenerator;

// Execution (js-runtime feature)
pub use ExecutionConfig, ExecutionPlan, ExecutionError;
pub use HttpExecutor, PlanCompiler, PlanExecutor;

// Tool builder
pub use CodeModeHandler, CodeModeToolBuilder;
```

## Proposed approach

**Option A: New crate in pmcp workspace** (recommended)

Add `pmcp-code-mode` as a new crate in the pmcp workspace, published to
crates.io alongside `pmcp`. This keeps the SDK modular — consumers opt in
via `pmcp-code-mode = { version = "0.1", features = ["js-runtime"] }`.

**Option B: Feature-gated module in pmcp**

Add `code_mode` as an optional module inside `pmcp` behind a `code-mode`
feature flag. Simpler for consumers (`pmcp = { features = ["code-mode"] }`)
but increases the main crate's dependency tree.

## Migration path

1. Copy `code_mode/` into the chosen location in pmcp
2. Update imports: `use mcp_server_common::code_mode::*` -> `use pmcp_code_mode::*`
3. Publish to crates.io
4. Update `mcp-server-common` to re-export from the new crate (backward compat)
5. Update downstream consumers (mcp-servers, pmcp-run built-in templates)

## Additional notes

- `mcp-server-common` currently specifies `pmcp = "1.9"` but the ecosystem
  uses 1.10. The pmcp dep should be bumped to 1.10 as part of this work.
- The remaining `mcp-server-common` modules (auth, secrets, config, prompts,
  resources) can stay private — they have AWS SDK dependencies and are specific
  to the pmcp-run commercial offering.

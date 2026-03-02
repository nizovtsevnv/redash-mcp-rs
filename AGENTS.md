# AGENTS.md — Instructions for AI Assistants

This file provides instructions for AI coding assistants (OpenCode, Claude Code, Cursor, Windsurf, etc.) working with the redash-mcp-rs codebase (MCP server for Redash).

## Sources of truth

- **PRD.md** — goals, architecture, data model, code and process requirements. Before implementing any feature or module — consult the PRD. If a task contradicts the PRD — ask the user, don't guess.
- **Source code** — current state of implementation. Don't assume what's implemented — read the code. Don't suggest changes to files you haven't read.

Everything else (stack, style, principles, phases, versioning) is described in PRD.md. Don't duplicate — reference.

## Workflow

1. Read PRD.md (in full or the relevant section)
2. Read the affected source files
3. Follow rules from the Restrictions section below
4. If PRD.md needs updating to reflect the new code state, request permission with a brief description of changes. Add only key aspects that describe the product from business logic, UX, and technical decision perspectives, without inflating context with technical minutiae. PRD is a concise structured specification — preserve this format when updating

## Dev environment

```bash
nix develop          # reproducible environment; automatically sets up git hooks
cargo check          # quick compilation check
cargo test           # tests
cargo clippy         # linter
```

## Restrictions

- Do not commit with `--no-verify`
- Do not use `unwrap()` in production code (tests only)
- Do not add dependencies without necessity
- Do not create abstractions "for the future"
- Code comments, doc comments (`///`), log messages, and commit messages must be in English
- Communicate with the user in their language (match the language of the user's messages)
- One commit = one logical unit with a clear message
- Tests go with the code, not "later"
- Pre-commit hooks are mandatory: `cargo fmt`, `cargo clippy`, `cargo test`

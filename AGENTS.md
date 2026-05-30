# Coding Guideline

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

# User Preference

- Communicate with the user in Traditional Chinese; keep technical terms in English. Code comments should be written in English.

# Project Guidelines

- Use [CONTEXT.md](CONTEXT.md) as the canonical product glossary. When naming domain concepts in code or docs, prefer the terms defined there; update it when a product term is resolved or renamed.
- Review existing ADRs in [docs/adr/](docs/adr/) before changing architecture, data retention, Tauri boundaries, provider strategy, or other hard-to-reverse decisions. Add a new ADR only when the decision is hard to reverse, surprising without context, and the result of a real trade-off.
- When adding features, consider the Tauri architecture boundary in [docs/tauri-architecture-boundary.md](docs/tauri-architecture-boundary.md).
- Follow [docs/dependency-management.md](docs/dependency-management.md) when installing or updating TypeScript/Rust dependencies.

# Project Commands

- Use Bun for project scripts: `bun run check`, `bun run tauri dev`, and `bun run tauri build`.
- Do not use npm/npx/pnpm/yarn for this project unless the user explicitly asks.
- For `src-tauri`, use Cargo directly from `src-tauri/`:
  - `cargo check --locked`
  - `cargo test --locked`
  - `cargo clippy --all-targets --all-features --locked -- -D warnings`

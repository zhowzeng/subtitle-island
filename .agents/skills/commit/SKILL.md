---
name: commit
description: Commit repository changes with a semantic Conventional Commits message. Use when the user asks to commit, says to use a semantic message, or wants a completed local-only git checkpoint.
---

# Semantic Commit

Use this workflow to create a clean semantic commit.

## Workflow

1. Inspect repository state:
   - `git status --short`
   - `git diff --stat`
   - `git diff --cached --stat`
2. Review unstaged and staged diffs enough to understand the change.
3. Decide whether the changes form one coherent commit.
   - Prefer multiple commits when the changes span distinct topics, workflows, or intentions.
   - If it is hard to describe the staged work with one concise Conventional Commits summary, split it into smaller commits.
4. Stage only files or hunks that belong to the current commit.
5. Choose a Conventional Commits message:
   - `docs:` for documentation, agent guides, README, wiki text.
   - `feat:` for user-visible new behavior.
   - `fix:` for bug fixes.
   - `refactor:` for behavior-preserving code changes.
   - `test:` for tests.
   - `chore:` for tooling, metadata, or maintenance.
6. Commit with `git commit -m "<type>: <summary>"`.
7. Repeat staging and committing for additional coherent groups when needed.
8. Report each commit hash and message.

## Guardrails

- Do not include unrelated user changes.
- Do not collapse unrelated or loosely related changes into one commit just because they were requested in the same session.
- Do not run remote git operations.
- Do not amend, squash, rebase, or reset unless the user explicitly asks.
- If the working tree has unexpected changes, explain what will be included before committing.
- If pre-commit hooks modify files or fail, inspect the result before retrying.
- Keep the commit message concise and imperative; do not add long bodies unless useful.


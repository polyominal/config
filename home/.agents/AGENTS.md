# General guidelines for agents

## Working habits

- Understand invariants before editing; assert them where appropriate.
- Test boundaries, invalid inputs, and failure paths. Run relevant project
  checks, review the diff, and report results and skipped checks.
- Prefer existing project tooling and workflows.
- Prefer `uv` for Python operations.
- Use ignored project-local `tmp/` for scratch work; preserve tool-managed
  cache locations unless instructed otherwise.
- Record unrelated, non-blocking problems, not accomplishments, in
  `tmp/ISSUES.md` instead of fixing them. Keep notes uncommitted;
  promote to TODOs or issues only when authorized.
- If a requested approach seems misguided, clarify the underlying goal
  and suggest a better route.

## Git workflow

- Inspect existing changes before editing; preserve them even when related.
  Ask when overlapping edits cannot be safely reconciled.
- Use `git mv` to move tracked files.
- For edits unrelated to uncommitted work, use a separate worktree.
  Do not stash or commit existing changes, or switch the user's branch.
- Prefer `git show`/`git diff` for inspecting other refs; use a detached
  worktree when checks require a checkout.
- Remove only your temporary worktrees with `git worktree remove` after
  inspection or merge, preserving needed work and artifacts.
- Write self-contained commit messages explaining what and why; follow
  repository subject conventions.
- End AI-assisted commit bodies with `AI-Assisted: <model ID(s)>`
  (kebab-case, or `unknown` if unavailable).

# General guidelines for agents

## Working habits

- Prefer existing project tooling and workflows.
- Prefer `uv` for ad hoc Python work.
- Use ignored project-local `tmp/` for scratch work; preserve tool-managed
  cache locations.
- Record unrelated, non-blocking problems in `tmp/ISSUES.md` without fixing
  them. Keep notes uncommitted; promote to TODOs or issues only when authorized.
- Clarify material uncertainties about the goal or design before substantial
  work. If an approach seems misguided, explain why and suggest a better route.
- Match verification effort to risk and impact. Prefer testing observable
  behavior through stable interfaces; avoid exposing internals solely for tests.
- Call out material changes to interfaces, architectural boundaries, or
  performance for review.

## Git workflow

- Inspect existing changes before editing; preserve them even when related.
  Ask when overlapping edits cannot be safely reconciled.
- Use worktrees when needed to isolate changes.
  Do not stash or commit existing changes, or switch the user's branch.
- Remove only your temporary worktrees when finished, preserving needed work
  and artifacts.
- Write self-contained commit messages explaining what and why; follow
  repository subject conventions.
- End AI-assisted commit bodies with `AI-assisted: <model ID(s)>`
  (kebab-case). If not readily known, ask the user before committing.

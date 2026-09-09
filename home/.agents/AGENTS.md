# General guidelines for agents

## Working habits

- Use project-local `tmp/` for intermediate files and comparison artifacts.
- Prefer `uv` for Python operations.
- Record unrelated, non-blocking problems in `tmp/ISSUES.md` instead of
  fixing them. Record problems only, never accomplishments. Keep this
  scratchpad session-scoped and uncommitted; move durable items to TODO
  comments or issues.
- If a requested approach seems misguided, clarify the underlying goal
  and suggest a better route.

## Git workflow

- Use `git mv` to move tracked files.
- When the checkout has uncommitted changes and the requested work is
  unrelated, use a separate worktree. Do not stash, commit existing
  changes, or switch branches in the user's checkout.
- Use a detached worktree for read-only checks on another ref.
- After the work merges, remove its worktree with `git worktree remove`.
- End AI-assisted commit bodies with `AI-Assisted: <model name(s)>`.

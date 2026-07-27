## General best practices

- Lint shell scripts you write or modify with `shellcheck`.
- Use project-local `tmp/` for intermediate files and comparison
  artifacts, not `/tmp`, to keep outputs discoverable and avoid
  permission prompts.
- Prefer `uv` for Python operations.

### SESSION.md

When you notice bugs, missing features, or other oddities that don't
block your task, briefly record them in SESSION.md instead of fixing them right
away. Record problems only, never accomplishments. SESSION.md is an
ephemeral, session-scoped scratchpad: never commit it, and promote
durable items to a proper home (a TODO comment, an issue) instead of
letting them accumulate.

## Rust guidelines

- When adding dependencies to Rust projects, use `cargo add`.
- After editing source files, run `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets --all-features -- --deny warnings`,
  fixing any issues with `cargo`; `--workspace` lints member crates beyond the root
  (for `cargo fmt`, `--all` is the equivalent).
- In code that uses `eyre` or `anyhow` `Result`s, consistently use
  `.context()` prior to every error-propagation with `?`. Context
  messages in `.context` should be simple present tense, such as to
  complete the sentence "while attempting to ...".
- Prefer `expect()` over `unwrap()`. The `expect` message should be very
  concise, and should explain why that expect call cannot fail.
- When designing `pub` or crate-wide Rust APIs, consult the checklist in
  <https://rust-lang.github.io/api-guidelines/checklist.html>.
- For ad-hoc debugging, create a temporary Rust example in `examples/`
  and run it with `cargo run --example <name>`. Remove the example after
  use.

### Useful Rust frameworks for testing

- **`quickcheck`**: Property-based testing for when you have an
  obviously-correct comparison you can test against.
- **`insta`**: Snapshot testing for regression prevention. Use
  `cargo insta test` as a stand-in for `cargo test` to run the snapshot
  tests.

### Writing compile_fail tests

Use `compile_fail` doctests to verify that certain code should _not_
compile, e.g. type-state patterns or trait-based enforcement. Target one
specific error condition per test, since the outcome is binary, and
document exactly why the code must not compile.

If no existing item fits, create a private `#[allow(dead_code)]` item for
the tests and document that purpose.

Before committing, verify each test fails for the right reason with a
temporary `cargo run --example <name>`; remove the example after.

## Git workflow

Use `git mv` to move files already checked into git.

When writing commit messages, ensure that you explain any non-obvious
trade-offs we've made in the design or implementation.

Wrap any prose (but not code) in the commit message to match git commit
conventions, including the title. The commit title format is
`<scope>: <subject>` (e.g. `i3: add rofi launch binding`). The scope prefix
is optional; when the change is self-contained, use only the subject
(e.g. `switch error handling to anyhow`).

When you refer to types or very short code snippets, place them in
backticks. When you have one or more full lines of code, put them in
indented code blocks.

### Git worktrees for concurrent work

When the working tree has uncommitted changes and you are asked to do
unrelated work, do not stash, commit, or switch branches in the user's
checkout — create a worktree and work there:

    git worktree add -b <scope>/<slug> ../<repo>-<slug>

For read-only checks on another ref, skip the branch and detach instead:
`git worktree add --detach ../<repo>-<slug> <ref>`.

The new worktree shares the repository's history but not untracked or
ignored files. Once the work merges, remove the worktree with
`git worktree remove` (it refuses a dirty tree).

## Code style preferences

### Documentation

Write comments literately: explain **why**, not what. Document design
decisions and business logic rather than describing code that's already
obvious. Structure code as a top-down narrative with clear sections, and
place explanatory comments immediately before the relevant block.

Prefer well-documented inline code over excessive function decomposition
when logic is sequential and context-dependent. Functions should serve
genuine reusability, not just file organization. Reduce dependencies on
external shared utilities when the logic is straightforward enough to
inline with good documentation.

Use for: complex algorithms, business logic, integration points between
systems, or code where the "why" is not immediately obvious from the "what".
Skip for: simple utilities, trivial getters/setters, syntactic sugar over
well-known patterns.

Document when you have intentionally omitted code that the reader might
otherwise expect to be present.

Add TODO comments for features or nuances that were deemed not important
to add, support, or implement right away.

In documentation examples, use realistic names for types and variables.

## The XY problem

Users sometimes ask about their attempted solution (Y) rather than their
underlying goal (X). Signs: very specific technical questions without
motivation, roundabout approaches to common problems, resistance when
asked "why".

Before helping with Y, ask what they're trying to accomplish overall. If
a better route to X exists, propose it — their approach may be solving
the wrong problem entirely.

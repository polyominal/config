## General best practices

- Run shell scripts through `shellcheck`.
- Use `tmp/` (project-local) for intermediate files and comparison
  artifacts, not `/tmp`. This keeps outputs discoverable and
  project-scoped, and avoids requesting permissions for `/tmp`.
- Prioritize doing Python operations using `uv`. If you are unsure of
  usage, consult its `--help`.

### SESSION.md

While working, if you come across any bugs, missing features, or other
oddities about the implementation, structure, or workflow, **add a
concise description of them to SESSION.md** to defer solving such
incidental tasks until later. You do not need to fix them all straight
away unless they block your progress; writing them down is often
sufficient. **Do not write your accomplishments into this file.**

## Rust guidelines

- When adding dependencies to Rust projects, use `cargo add`.
- After editing source files, ensure formatting and linting are correct
  by running `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  and fix using `cargo` commands if there are issues. `--workspace` is
  required for clippy to lint member crates beyond the workspace root
  (for `cargo fmt`, `--all` is the correct equivalent flag).
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

### Writing compile_fail Tests

Use `compile_fail` doctests to verify when certain code should _not_
compile, such as for type-state patterns or trait-based enforcement.
Each `compile_fail` test should target a specific error condition since
the doctest only has a binary output of whether it fails to compile, not
the many reasons _why_. Make sure you clearly explain exactly WHY the
code should fail to compile.

If there is no obvious item to add the doctest to, create a new private
item with `#[allow(dead_code)]` that you add the compile-fail tests to.
Document that that's its purpose.

Before committing, create a temporary example file for each
compile-fail test and check the output of `cargo run --example <name>`
to ensure it fails for the correct reason. Remove the temporary example
after.

## Git workflow

Make sure you use `git mv` to move any files that are already checked into
git.

When writing commit messages, ensure that you explain any non-obvious
trade-offs we've made in the design or implementation.

Wrap any prose (but not code) in the commit message to match git commit
conventions, including the title. The commit title format is
`<scope>: <subject>` (e.g. `i3: add rofi launch binding`). The scope prefix
is optional; when the change is self-contained, use only the subject
(e.g. `switch error handling to anyhow`).

**When you refer to types or very short code snippets, place them in backticks**.
When you have a full line of code or more than one line of code,
put them in indented code blocks.

## Documentation preferences

### Documentation examples

- Use realistic names for types and variables.

## Code style preferences

Document when you have intentionally omitted code that the reader might
otherwise expect to be present.

Add TODO comments for features or nuances that were deemed not important
to add, support, or implement right away.

### Literate Programming

Explain **why**, not what. Document design decisions and business logic
rather than describing code that's already obvious. Structure code as a
top-down narrative with clear sections, and place explanatory comments
immediately before the relevant block.

Prefer well-documented inline code over excessive function decomposition
when logic is sequential and context-dependent. Functions should serve
genuine reusability, not just file organization. Reduce dependencies on
external shared utilities when the logic is straightforward enough to
inline with good documentation.

Use for: complex algorithms, business logic, integration points between
systems, or code where the "why" is not immediately obvious from the "what".

Skip for: simple utilities, trivial getters/setters, syntactic sugar over
well-known patterns.

## Common failure modes when helping

### The XY Problem

The XY problem occurs when someone asks about their attempted solution
(Y) instead of their actual underlying problem (X).

#### The Pattern
1. User wants to accomplish goal X
2. User thinks Y is the best approach to solve X
3. User asks specifically about Y, not X
4. Helper becomes confused by the odd/narrow request
5. Time is wasted on suboptimal solutions

#### Warning Signs to Watch For
- Focus on a specific technical method without explaining why
- Resistance to providing broader context when asked
- Rejecting alternative approaches outright
- Questions that seem oddly narrow or convoluted
- "How do I get the last 3 characters of a filename?" (when they want
  file extension)

#### How to Avoid It (As Helper)
- **Ask probing questions**: "What are you trying to accomplish overall?"
- **Request context**: "Can you explain the bigger picture?"
- **Challenge assumptions**: "Why do you think this approach will work?"
- **Offer alternatives**: "Have you considered...?"

#### Red Flags in User Requests
- Very specific technical questions without motivation
- Unusual or roundabout approaches to common problems
- Dismissal of "why do you want to do that?" questions
- Focus on implementation details before problem definition

#### Key Principle
Always try to understand the fundamental problem (X) before helping with
the proposed solution (Y). The user's approach may not be optimal or may
indicate they're solving the wrong problem entirely.

# Agent Rules
## Implementation Principles
- Tests must always pass or must be adjusted to reflect code changes.
- Ensure the application builds before completing a task.
- When refactoring, NO feature must be lost. Ensure parity.
- WE DO NOT SUPPORT comparison operators like <, >, <=, >= in the grammar. Use ranges (min ... max) instead.
- Never change the Hippocrates Engine to fix failures that originate in the app; fix the app instead.

## Repository Handling
- All GitHub issues and pull requests MUST be authored in Markdown with headings and bullet lists; use code fences where needed for clarity.
- Commit messages must use a Conventional Commits summary line with no Markdown prefixes (no `#`, `##`, or `-`). If a body is needed, add it after a blank line and use Markdown bullet lists.
- Do not submit or leave GitHub artifacts in plain-text form; reformat to structured Markdown before posting or updating.

- Commits: Conventional style observed in history, e.g. `feat(api): ...`, `fix(tests): ...`, `refactor(agents): ...`, `chore(infra): ...` with concise imperative subject. Do not prefix the subject with Markdown headings or bullets; keep the first line parseable.
- PRs: Include scope, motivation, and test notes. Link issues; add screenshots for UI. Ensure `make test` passes; for API changes, run `make proto` and include generated diffs.
- PR titles and summaries must describe the entire change set in the PR, not just the latest commit, so reviewers can understand all modifications at a glance.

- Whenever a pull request is created for a tracked GitHub issue, the PR must explicitly link the issue (e.g., `Closes #<issue>` in the PR body). This applies to all PRs opened by agents and humans.

# Agent Guide

Read these before starting work:

- [`MISSION.md`](MISSION.md): what the game is and the principles every change must respect.
- [`architecture/`](architecture/): Architecture Decision Records (ADRs). Follow the
  Accepted ones. Treat "Proposed" ones as open questions for the user. Never mark an
  ADR Accepted yourself; only the user does that.
- [`README.md`](README.md): dev setup, run, and test commands.

## Task tracking: Gitea issues

Tasks live in the issue tracker of the `fluid-game` repo on the local Gitea
(`http://localhost:3000`).

- **Pick:** before starting work, pick an open issue.
- **Claim:** assign it to yourself or the user before starting.
- **Split:** if an issue is too big for one tracer-bullet slice, split it into new
  issues that link back to it. Don't work an oversized issue as a whole.
- **Discover:** when you find new work mid-task, open a new issue for it right away
  instead of widening the current one.
- **Finish:** close an issue only after its testable target passes. Add a one-line
  note saying what proved it.

## Workflow: tracer bullets

Ship every change as a small, self-contained slice that runs end-to-end and proves
itself with a concrete test before the next slice starts.

1. **State the target first.** Before coding a slice, write down its testable
   target, e.g. "an enclosed blob of player-2 fluid becomes player-1 after one
   tick; unit test passes".
2. **End-to-end.** Each slice runs through the whole path it touches (input →
   simulation → render, or whichever part applies), even if each stage is minimal.
3. **Verify before stacking.** Get one slice working and tested before starting
   the next.
4. **Split up front.** If work doesn't fit one slice, break it into an ordered
   list of slices (as linked issues) before writing code.

## Commits and pull requests

- Commit each finished, verified slice without asking first, but **never push
  to `main`**. Branch off `main`, commit and push to the branch, then open a PR
  on Gitea that references its issue (e.g. `Closes #12`). Leave the PR open
  for the user to review and merge. Never merge it yourself.
- Before committing new or changed architecture decisions, ask the user to
  review and accept them first.
- **Test instructions:** after pushing, give the user what they need to test
  the change themselves, both in the PR description and in your reply:
  - the branch to check out;
  - the exact commands to run (e.g. `cargo run`, `cargo test -p <crate>`);
  - where a command can't prove it, the manual test steps: what to do in the
    game and what they should see.

### Gitea access

- Repo: `http://localhost:3000/szyx/fluid-game` (remote `origin`).
- The API token is in `.secrets/gitea_token` (gitignored). Read it from the file
  whenever you need it (`Authorization: token …`). Never print it, commit it,
  or paste it into URLs, issues, or PRs.
- `git push` authenticates through a repo-local credential helper that reads
  the same file.

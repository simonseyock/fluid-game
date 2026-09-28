# Architecture Decision Records

This folder holds numbered ADRs, one decision per file, named `NNNN-short-title.md`
(e.g. `0001-native-language.md`). Numbers are assigned in order and never reused.

Each ADR has these sections:

- **Status:** `Proposed — pending user review`, `Accepted`, `Superseded by NNNN`,
  or `Rejected`
- **Context:** the problem and the forces at play
- **Decision:** what we will do
- **Rationale:** why this option over the alternatives
- **Consequences:** what follows, good and bad

## Rules

- Every new or edited ADR gets the status **"Proposed — pending user review"**.
- Only the user marks an ADR **Accepted**. Agents never approve their own decisions.
- To change an Accepted decision, write a new ADR that supersedes it. Don't rewrite
  history.

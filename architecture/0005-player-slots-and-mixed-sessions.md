# 0005. N player slots and mixed local/network sessions

**Status:** Accepted

## Context

The game is played by two or more players. They can sit at one computer (two on
the keyboard, one on the mouse), play from separate devices over the network, or
mix both (e.g. two on a laptop keyboard vs. one on Android).

The simulation core (ADR 0002), grid (ADR 0003), and input sources (ADR 0004)
therefore need a single model of who the players are and where their input
comes from.

## Decision

### Players

- A match has **N player slots**, N ≥ 2, picked in the level setup screen.
  The maximum is a compile-time constant, starting at 4 and raised only if
  colors, controls, and balance hold up. On one desktop, at most 3 slots are
  local (ADR 0004). Any further players join over the network.
- A `PlayerId` is a small integer (`0..N`). Grid cells store their owner as a
  `PlayerId`.
- Each slot gets a distinct neon color from a fixed palette chosen so every
  pair is easy to tell apart on the dark background. The palette size caps N.

### Rules

- **Encirclement:** the exact rules, including enclosures formed by several
  players together, are defined in a separate ADR, not here. This ADR only
  requires that they work for any number of players.
- **Elimination and winning:** a player whose color hits zero cells is out.
  The last color left wins. When time runs out, the player with the most
  cells wins (ties stay ties).

### Sessions

- A **session** is a list of peers (devices). Each peer owns zero or more
  slots, and every slot has exactly one owner.
  - One peer owning all slots = a purely local match. No network needed.
  - Several peers = a network match. Each peer runs the same simulation from
    all slots' commands (ADR 0002).
- The network layer sends commands per **slot**, not per device. A rollback
  library that supports several local players per peer fits this (GGRS does).
- Platform limits come from ADR 0004 (e.g. Android owns exactly one slot).

### Local controls on one desktop

- Up to **two keyboard players**, each with a **key zone** (a group of keys
  for moving its cursor). Defaults are WASD (left side) and the arrow keys
  (right side). Key zones can be changed in the setup screen.
- Up to **one mouse player**. The mouse sits to the right of the keyboard, so
  the second keyboard zone may need moving (e.g. IJKL) if the arrow-key player
  and the mouse player crowd each other. Configurable, not fixed.
- In the setup screen, each local slot is assigned an input device:
  keyboard zone 1, keyboard zone 2, or mouse.

## Rationale

Treating players as slots with owners (local or remote) makes local, network,
and mixed matches the same thing internally. The simulation sees only "N
players' commands per tick". Local-only is the special case with one peer, so
it needs no separate code path. Per-slot commands match how rollback netcode
already works.

## Consequences

- No code may assume a fixed player count. Loops, arrays, and UI iterate over
  the match's slots.
- **Keyboard ghosting/rollover:** cheap keyboards may drop key presses when
  both keyboard players press several keys at once. The default zones sit at
  opposite ends of the keyboard to lower that risk.
- Balance across player counts (map size, spawn positions, fluid per player)
  needs testing. Maps may need a supported player-count range.

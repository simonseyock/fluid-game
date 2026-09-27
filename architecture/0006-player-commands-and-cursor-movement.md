# 0006. Player commands and simulation-owned cursor movement

**Status:** Accepted

## Context

Players steer with different devices: keyboard key zones, the mouse, and later
touch on Android (ADRs 0004, 0005). These produce very different raw input:

- A **keyboard** gives held directions (up/down/left/right, 8 directions).
- A **mouse** or **touch** gives an absolute position that can jump across the
  whole screen in one frame.

ADR 0002 syncs only per-tick player commands over the network, so the command
format is part of the network protocol. MISSION.md also requires fairness:
no device may move its cursor faster or more freely than the rules allow.

## Decision

- **One command type for all devices:** each tick, each player slot sends
  exactly one command:
  - `Idle`: no input.
  - `Direction(dx, dy)` with `dx, dy ∈ {-1, 0, 1}`, from keyboard zones.
  - `Target(x, y)` in **grid cell coordinates** (integers), from mouse or touch.
- **Commands are fixed-size and plain data,** so they can go straight into a
  rollback library (GGRS needs this).
- **The simulation owns cursor positions.** Cursor positions are part of the
  game state. Each tick the simulation moves each cursor by at most a **maximum
  cursor speed** (cells per tick, the same for everyone):
  - `Direction` moves the cursor that way.
  - `Target` moves the cursor toward the target, never jumping there.
  So a mouse player can aim anywhere but can't teleport.
- **Screen-to-grid conversion happens in the input layer.** The mouse input
  source converts the pointer's window position into a grid cell using the
  grid size chosen in the setup screen (ADR 0003) and how that grid is
  currently scaled into the window, and clamps it to the grid.
  The simulation only ever sees grid coordinates.

## Rationale

- Keeping cursors in the simulation state makes cursor movement a game rule, so
  it's deterministic and identical on every peer. The cap enforces fairness
  between devices in one place.
- One small command type keeps the network protocol simple, and mouse and touch
  share the same path.
- Grid coordinates make commands independent of each peer's window size and
  screen resolution.

## Consequences

- Balance between devices needs playtesting. A mouse gives precise aim in any
  direction, while a keyboard has 8 directions but no need to hold a hand
  still. The max cursor speed and possibly analog-style diagonals for keyboards
  are the tuning knobs.
- The grid size is chosen in the setup screen and fixed per match (ADR 0003),
  so grid coordinates stay valid throughout a match and mean the same cell on
  every peer. Each device only decides how large a cell is drawn in its own
  window.
- Cursor speed and position math must use integers or fixed-point
  (ADR 0002, ADR 0004).

# 0002. Deterministic simulation core, separated from input and rendering

**Status:** Accepted

## Context

The game ships as local multiplayer on one keyboard, but MISSION.md requires that
networked play can be added later. The fluid simulation, the encirclement rule
(enclosed enemy fluid changes color), and the win condition (last color standing,
or decided when time runs out) are the whole game state.

## Decision

Structure the game as three layers:

1. **Simulation core:** a pure, deterministic step function
   `state' = step(state, inputs_for_this_tick)` running at a fixed tick rate.
   It covers fluid movement toward cursors, obstacle collision, encirclement
   and color swaps, and the match clock and win check. It has no rendering,
   no windowing, and no wall-clock time.
2. **Input sources:** turn raw input into per-player, per-tick commands. The
   first one splits the shared keyboard into commands for each local player. A network
   source can be added later behind the same interface.
3. **Renderer:** draws a read-only view of the state as a pixel grid in neon
   colors on a dark background.

## Rationale

A deterministic, input-driven core lets a future network mode sync only player
inputs (lockstep or rollback) instead of the full fluid state. It also makes
game rules unit-testable without a window, which suits the tracer-bullet workflow.

## Consequences

- No floating-point behavior that varies between platforms in the core
  (or use fixed-point math) if cross-machine lockstep is ever needed.
- No random-number generator the core doesn't control. Randomness comes from
  a seed stored in the state.
- Frame rate and tick rate are decoupled. The renderer may interpolate.

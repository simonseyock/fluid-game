# 0003. Pixel-grid world with configurable grid size

**Status:** Accepted

## Context

The game is rendered in pixels, and the grid size is set in the level setup
screen. Maps contain obstacles. The encirclement rule needs a clear definition
of what "enclosed" means.

## Decision

- The world is a 2D grid of cells. Each cell is empty, an obstacle, or holds
  fluid owned by one of the players.
- The **grid size** (width × height in cells) is chosen in the level setup
  screen and stays fixed during a match. It's part of the game state, so
  every peer in a network match simulates the same grid.
- Each cell is drawn as one "game pixel". How many screen pixels a cell takes
  is decided per device at render time, by scaling the grid to fit the window.
  This never affects the simulation.
- Encirclement is computed on the grid. Enemy fluid is enclosed when it can't
  reach any cell outside the encircling player's boundary. Flood fill is the
  first candidate; whether obstacles and map edges count as part of an enclosure
  is still to decide.

## Rationale

A discrete grid keeps the simulation deterministic (ADR 0002), maps directly onto
the pixel look, and turns encirclement into a well-defined connectivity problem.

## Consequences

- Larger grids mean more cells and more simulation cost. The allowed
  grid-size range needs a performance check.
- Scaling a grid into a window with a different aspect ratio leaves margins
  (letterboxing). They're filled with the dark background.
- Cells may not map to a whole number of screen pixels. Whether to round to
  whole-pixel scaling (sharper, bigger margins) or allow fractional scaling
  is a rendering detail for a later slice.
- Open questions: how obstacles and map edges count toward encirclement, and
  4- vs 8-connectivity.

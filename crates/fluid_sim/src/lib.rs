//! Deterministic simulation core (ADR 0002).
//!
//! `step(state, commands)` advances the match by one tick. Everything here is
//! integer math over plain data: no engine, no wall-clock time, no floats, so the
//! same inputs give the same state on every machine.

mod command;
mod distance;
mod encircle;
mod grid;
mod map;
mod state;

pub use command::Command;
pub use grid::{Cell, Grid, PlayerId};
pub use map::{Map, Rect};
pub use state::{Cursor, GameState, MatchConfig, SUBCELL};

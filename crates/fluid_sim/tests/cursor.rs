//! Cursor movement: capped speed, grid bounds, and targets (ADR 0006).

mod common;

use common::small;
use fluid_sim::{Command, Cursor, GameState};

#[test]
fn cursor_moves_at_capped_speed_and_stays_on_grid() {
    let mut s = GameState::new(small());
    let start = s.cursors[0];
    s.step(&[Command::Direction { dx: -1, dy: 0 }]);
    assert_eq!(s.cursors[0].x, start.x - s.config.cursor_speed);
    for _ in 0..1000 {
        s.step(&[Command::Direction { dx: -1, dy: -1 }]);
    }
    assert_eq!(s.cursors[0].cell(), (0, 0));
}

#[test]
fn target_moves_cursor_gradually() {
    let mut s = GameState::new(small());
    let start = s.cursors[0];
    s.step(&[Command::Target { x: 39, y: 29 }]);
    assert_moved_one_step(&s, start);
    for _ in 0..1000 {
        s.step(&[Command::Target { x: 39, y: 29 }]);
    }
    assert_eq!(s.cursors[0].cell(), (39, 29));
}

#[test]
fn diagonal_is_not_faster_than_straight() {
    let mut s = GameState::new(small());
    let start = s.cursors[0];
    s.step(&[Command::Direction { dx: 1, dy: 1 }]);
    assert_eq!(s.cursors[0].x - start.x, s.cursors[0].y - start.y);
    assert_moved_one_step(&s, start);
}

/// Player 0's cursor moved a full step of `cursor_speed` from `start`, measured
/// as straight-line length (integer rounding may lose a unit or two).
fn assert_moved_one_step(s: &GameState, start: Cursor) {
    let (dx, dy) = (s.cursors[0].x - start.x, s.cursors[0].y - start.y);
    let moved = f64::from(dx * dx + dy * dy).sqrt();
    let speed = f64::from(s.config.cursor_speed);
    assert!(
        moved <= speed && moved > speed - 2.0,
        "moved {moved}, speed {speed}"
    );
}

//! Cursor movement: capped speed, grid bounds, and targets (ADR 0006).

mod common;

use common::small;
use fluid_sim::{Command, GameState};

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
    assert_eq!(s.cursors[0].x - start.x, s.config.cursor_speed);
    for _ in 0..1000 {
        s.step(&[Command::Target { x: 39, y: 29 }]);
    }
    assert_eq!(s.cursors[0].cell(), (39, 29));
}

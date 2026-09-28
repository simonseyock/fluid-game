//! Fluid spawning and movement.

mod common;

use common::small;
use fluid_sim::{Cell, Command, GameState, MatchConfig, PlayerId, SUBCELL};

/// Sum of Manhattan distances from each of player `p`'s fluid cells to their cursor.
fn total_distance(state: &GameState, p: usize) -> u64 {
    let (cx, cy) = state.cursors[p].cell();
    let g = &state.grid;
    (0..g.height())
        .flat_map(|y| (0..g.width()).map(move |x| (x, y)))
        .filter(|&(x, y)| g.get(x, y) == Cell::Fluid(p as PlayerId))
        .map(|(x, y)| (x.abs_diff(cx) + y.abs_diff(cy)) as u64)
        .sum()
}

#[test]
fn new_match_gives_each_player_equal_fluid() {
    let s = GameState::new(MatchConfig {
        player_count: 3,
        ..small()
    });
    assert!(s.fluid_count(0) > 0);
    assert_eq!(s.fluid_count(0), s.fluid_count(1));
    assert_eq!(s.fluid_count(1), s.fluid_count(2));
}

#[test]
fn fluid_flows_toward_its_moved_cursor() {
    let mut s = GameState::new(small());
    // Put player 0's cursor away from its blob, then let the fluid follow.
    s.cursors[0].x += 12 * SUBCELL;
    let before = total_distance(&s, 0);
    for _ in 0..30 {
        s.step(&[Command::Idle, Command::Idle]);
    }
    assert!(
        total_distance(&s, 0) < before / 2,
        "fluid should close in on the cursor"
    );
}

#[test]
fn fluid_is_conserved() {
    let mut s = GameState::new(small());
    let counts = (s.fluid_count(0), s.fluid_count(1));
    for t in 0..200 {
        let dy = if t % 50 < 25 { 1 } else { -1 };
        s.step(&[
            Command::Direction { dx: 1, dy },
            Command::Direction { dx: -1, dy: -dy },
        ]);
    }
    assert_eq!((s.fluid_count(0), s.fluid_count(1)), counts);
}

#[test]
fn same_inputs_give_same_state() {
    let script = |t: u64| {
        [
            Command::Direction {
                dx: (t % 3) as i8 - 1,
                dy: 1,
            },
            Command::Target {
                x: (t % 40) as u16,
                y: 3,
            },
        ]
    };
    let (mut a, mut b) = (GameState::new(small()), GameState::new(small()));
    for t in 0..300 {
        a.step(&script(t));
        b.step(&script(t));
    }
    assert_eq!(a, b);
}

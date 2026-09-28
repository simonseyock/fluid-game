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

/// Average position of player 0's fluid, in cells.
fn centroid(s: &GameState) -> (f64, f64) {
    let g = &s.grid;
    let cells: Vec<(usize, usize)> = (0..g.height())
        .flat_map(|y| (0..g.width()).map(move |x| (x, y)))
        .filter(|&(x, y)| g.get(x, y) == Cell::Fluid(0))
        .collect();
    let n = cells.len() as f64;
    let sum = cells
        .iter()
        .fold((0, 0), |(sx, sy), &(x, y)| (sx + x, sy + y));
    (sum.0 as f64 / n, sum.1 as f64 / n)
}

/// Steer player 0 with `command` and check its fluid covers clearly less
/// ground than its cursor over a stretch of steady movement.
fn assert_fluid_slower(command: Command) {
    let mut s = GameState::new(MatchConfig {
        width: 160,
        height: 160,
        ..MatchConfig::default()
    });
    let commands = [command, Command::Idle];
    // Get going, then measure.
    for _ in 0..40 {
        s.step(&commands);
    }
    let (cursor_before, fluid_before) = (s.cursors[0], centroid(&s));
    for _ in 0..80 {
        s.step(&commands);
    }
    let (cdx, cdy) = (
        f64::from(s.cursors[0].x - cursor_before.x) / f64::from(SUBCELL),
        f64::from(s.cursors[0].y - cursor_before.y) / f64::from(SUBCELL),
    );
    let fluid_after = centroid(&s);
    let (fdx, fdy) = (
        fluid_after.0 - fluid_before.0,
        fluid_after.1 - fluid_before.1,
    );
    let (cursor_cells, fluid_cells) = (cdx.hypot(cdy), fdx.hypot(fdy));
    assert!(
        fluid_cells < 0.8 * cursor_cells,
        "fluid moved {fluid_cells:.1} cells while the cursor moved {cursor_cells:.1}"
    );
}

#[test]
fn fluid_moves_slower_than_the_cursor() {
    assert_fluid_slower(Command::Direction { dx: 1, dy: 0 });
}

#[test]
fn fluid_moves_slower_than_the_cursor_diagonally() {
    assert_fluid_slower(Command::Direction { dx: 1, dy: 1 });
}

//! Encirclement, prototype rule (issue #5): a fluid cell flips when more of
//! its 8 neighbours are opponents than its own colour. It flips to the most
//! common opponent; a tie between opponents means no flip. Empty cells and
//! obstacles don't count. All cells decide from the same snapshot.

use fluid_sim::{Cell, Command, GameState, Grid, MatchConfig, SUBCELL};

const A: Cell = Cell::Fluid(0);
const B: Cell = Cell::Fluid(1);
const C: Cell = Cell::Fluid(2);

/// Build a grid from rows of characters: `a`, `b`, `c` fluid, `#` obstacle,
/// `.` empty.
fn grid(rows: &[&str]) -> Grid {
    let mut g = Grid::new(rows[0].len(), rows.len());
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let cell = match ch {
                'a' => A,
                'b' => B,
                'c' => C,
                '#' => Cell::Obstacle,
                _ => Cell::Empty,
            };
            g.set(x, y, cell);
        }
    }
    g
}

#[test]
fn flat_front_is_stable() {
    let mut g = grid(&["aaabbb", "aaabbb", "aaabbb", "aaabbb"]);
    let before = g.clone();
    assert_eq!(g.flip_outnumbered(), 0);
    assert_eq!(g, before);
}

#[test]
fn a_bulge_into_the_enemy_flips() {
    let mut g = grid(&["aaabbb", "aaaabb", "aaabbb"]);
    g.flip_outnumbered();
    // The bulging cell sees 5 opponents and 3 of its own.
    assert_eq!(g.get(3, 1), B);
}

#[test]
fn empty_cells_and_obstacles_do_not_count() {
    // A lone cell next to one opponent: 1 opponent vs 0 own, so it flips.
    let mut lonely = grid(&["...", ".ab", "..."]);
    lonely.flip_outnumbered();
    assert_eq!(lonely.get(1, 1), B);
    // Surrounded by walls and one friend, facing one opponent: 1 vs 1, stays.
    let mut walled = grid(&["###", "aab", "###"]);
    walled.flip_outnumbered();
    assert_eq!(walled.get(1, 1), A);
}

#[test]
fn all_cells_decide_from_the_same_snapshot() {
    // Two lone cells facing each other: each sees 1 opponent and 0 own.
    // Deciding together, they both flip (swap), rather than the first flip
    // saving the second.
    let mut g = grid(&["....", ".ab.", "...."]);
    g.flip_outnumbered();
    assert_eq!((g.get(1, 1), g.get(2, 1)), (B, A));
}

#[test]
fn flips_to_the_most_common_opponent_and_ties_do_not_flip() {
    // The centre `c` sees 5 a, 2 b, 1 c: flips to a.
    let mut majority = grid(&["aaa", "acb", "acb"]);
    majority.flip_outnumbered();
    assert_eq!(majority.get(1, 1), A);
    // The centre `c` sees 3 a, 3 b, 2 c: opponents tie, no flip.
    let mut tie = grid(&["aab", "cca", "bcb"]);
    tie.flip_outnumbered();
    assert_eq!(tie.get(1, 1), C);
}

#[test]
fn small_enclosed_blob_is_eaten_layer_by_layer() {
    let mut g = grid(&[
        "aaaaaaa", "aaaaaaa", "aabbbaa", "aabbbaa", "aabbbaa", "aaaaaaa", "aaaaaaa",
    ]);
    let total = g.count(A) + g.count(B);
    let mut rounds = 0;
    while g.flip_outnumbered() > 0 {
        rounds += 1;
        assert!(rounds < 10, "should settle quickly");
    }
    assert_eq!(g.count(B), 0);
    assert_eq!(g.count(A), total);
    // Corners first, then edges, then the centre: more than one round.
    assert!(rounds > 1);
}

#[test]
fn encirclement_happens_during_a_match() {
    let mut s = GameState::new(MatchConfig {
        width: 40,
        height: 30,
        ..MatchConfig::default()
    });
    // Replace the spawns with a small b blob inside a thick ring of a.
    for y in 0..30 {
        for x in 0..40 {
            let (dx, dy) = (x as i32 - 20, y as i32 - 15);
            let cell = match dx.abs().max(dy.abs()) {
                0..=1 => B,
                2..=5 => A,
                _ => Cell::Empty,
            };
            s.grid.set(x, y, cell);
        }
    }
    // Both cursors in the middle, so neither blob flows away.
    for cursor in &mut s.cursors {
        cursor.x = 20 * SUBCELL + SUBCELL / 2;
        cursor.y = 15 * SUBCELL + SUBCELL / 2;
    }
    let total = s.fluid_count(0) + s.fluid_count(1);
    for _ in 0..60 {
        s.step(&[Command::Idle, Command::Idle]);
    }
    assert_eq!(s.fluid_count(1), 0);
    assert_eq!(s.fluid_count(0), total);
}

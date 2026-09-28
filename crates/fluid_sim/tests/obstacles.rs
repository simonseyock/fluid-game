//! Obstacles: maps scale with the grid, fluid routes around walls and never
//! enters them (ADR 0003).

use fluid_sim::{Cell, Command, GameState, Map, MatchConfig, Rect, SUBCELL};

/// A wall down the middle, open only at the bottom.
const WALL_WITH_GAP: Map = Map {
    name: "wall with gap",
    obstacles: &[Rect {
        x0: 7,
        y0: 0,
        x1: 9,
        y1: 12,
    }],
};

fn config(map: Map, size: usize) -> MatchConfig {
    MatchConfig {
        width: size,
        height: size,
        map,
        ..MatchConfig::default()
    }
}

fn share_of_fluid_right_of(s: &GameState, x_min: usize) -> f64 {
    let g = &s.grid;
    let (mut right, mut total) = (0, 0);
    for y in 0..g.height() {
        for x in 0..g.width() {
            if g.get(x, y) == Cell::Fluid(0) {
                total += 1;
                right += usize::from(x >= x_min);
            }
        }
    }
    right as f64 / total as f64
}

#[test]
fn obstacles_scale_with_the_grid() {
    let small = GameState::new(config(WALL_WITH_GAP, 80));
    let large = GameState::new(config(WALL_WITH_GAP, 160));
    // 2/16 × 12/16 of the grid.
    assert_eq!(small.grid.count(Cell::Obstacle), 10 * 60);
    assert_eq!(large.grid.count(Cell::Obstacle), 20 * 120);
}

#[test]
fn spawns_leave_obstacles_alone_and_stay_fair() {
    for players in 2..=4 {
        let s = GameState::new(MatchConfig {
            player_count: players,
            map: Map::PILLARS,
            ..MatchConfig::default()
        });
        let expected = GameState::new(MatchConfig {
            player_count: 2,
            map: Map::PILLARS,
            ..MatchConfig::default()
        })
        .grid
        .count(Cell::Obstacle);
        assert_eq!(s.grid.count(Cell::Obstacle), expected);
        let first = s.fluid_count(0);
        assert!(first > 0);
        for p in 1..players {
            assert_eq!(s.fluid_count(p as u8), first, "{players} players");
        }
    }
}

#[test]
fn fluid_routes_through_the_gap_and_never_enters_the_wall() {
    let mut s = GameState::new(config(WALL_WITH_GAP, 80));
    let walls = s.grid.count(Cell::Obstacle);
    // Cursor on the far side of the wall, near the top: the only way there
    // is down through the gap at the bottom and back up.
    s.cursors[0].x = 60 * SUBCELL + SUBCELL / 2;
    s.cursors[0].y = 12 * SUBCELL + SUBCELL / 2;
    for _ in 0..900 {
        s.step(&[Command::Idle, Command::Idle]);
        assert_eq!(s.grid.count(Cell::Obstacle), walls);
    }
    let share = share_of_fluid_right_of(&s, 45);
    assert!(share >= 0.9, "only {share:.2} of the fluid made it through");
}

#[test]
fn cursor_over_a_wall_pulls_fluid_to_the_wall() {
    let mut s = GameState::new(config(WALL_WITH_GAP, 80));
    // Cursor in the middle of the wall, level with player 0's blob.
    s.cursors[0].x = 40 * SUBCELL + SUBCELL / 2;
    s.cursors[0].y = 40 * SUBCELL + SUBCELL / 2;
    for _ in 0..300 {
        s.step(&[Command::Idle, Command::Idle]);
    }
    // Fluid piles up against the wall's left face (x = 34).
    assert_eq!(s.grid.get(34, 40), Cell::Fluid(0));
}

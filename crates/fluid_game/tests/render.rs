//! Painting the grid texture: one RGBA texel per cell, cursors on top in
//! their player's colour with a black outline.

use fluid_game::render::{PLAYER_COLORS, paint};
use fluid_sim::{Cell, GameState, MatchConfig, SUBCELL};

const BLACK: [u8; 4] = [0, 0, 0, 255];

fn texel(data: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let i = (y * width + x) * 4;
    data[i..i + 4].try_into().unwrap()
}

fn state() -> GameState {
    GameState::new(MatchConfig {
        width: 40,
        height: 30,
        player_count: 3,
        ..MatchConfig::default()
    })
}

fn painted(s: &GameState) -> Vec<u8> {
    let mut data = vec![0; s.grid.width() * s.grid.height() * 4];
    paint(s, &mut data);
    data
}

#[test]
fn cells_are_painted_in_their_colours() {
    let mut s = state();
    s.grid.set(0, 0, Cell::Obstacle);
    s.grid.set(1, 0, Cell::Fluid(1));
    let data = painted(&s);
    assert_eq!(texel(&data, 40, 1, 0), PLAYER_COLORS[1]);
    assert_ne!(texel(&data, 40, 0, 0), BLACK, "obstacles aren't background");
    assert_eq!(texel(&data, 40, 39, 29), BLACK, "empty is background");
}

#[test]
fn each_cursor_is_a_cross_in_its_players_colour() {
    let s = state();
    let data = painted(&s);
    for (p, cursor) in s.cursors.iter().enumerate() {
        let (x, y) = cursor.cell();
        for (cx, cy) in [(x, y), (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            assert_eq!(texel(&data, 40, cx, cy), PLAYER_COLORS[p], "player {p}");
        }
    }
}

#[test]
fn a_cursor_over_its_own_fluid_has_a_black_outline() {
    // Spawned cursors sit in the middle of their own blob.
    let s = state();
    let data = painted(&s);
    let (x, y) = s.cursors[0].cell();
    assert_eq!(
        s.grid.get(x + 2, y),
        Cell::Fluid(0),
        "own fluid under the outline"
    );
    for (ox, oy) in [
        (x + 2, y),
        (x - 2, y),
        (x, y + 2),
        (x, y - 2),
        (x + 1, y + 1),
        (x - 1, y - 1),
    ] {
        assert_eq!(texel(&data, 40, ox, oy), BLACK);
    }
}

#[test]
fn cursors_at_the_grid_edge_are_clipped() {
    let mut s = state();
    s.cursors[0].x = 0;
    s.cursors[0].y = 0;
    s.cursors[1].x = 40 * SUBCELL - 1;
    s.cursors[1].y = 30 * SUBCELL - 1;
    let data = painted(&s);
    assert_eq!(texel(&data, 40, 0, 0), PLAYER_COLORS[0]);
    assert_eq!(texel(&data, 40, 39, 29), PLAYER_COLORS[1]);
}

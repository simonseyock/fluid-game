//! Every built-in map is fair and playable at every allowed grid size
//! (ADR 0003): spawns are free of obstacles, no open area is sealed off, and
//! the layout is mirror-symmetric.

use fluid_sim::{Cell, GameState, Map, MatchConfig, Rect};

/// Grid sizes at and between the setup screen's limits (60–240 × 40–135).
const SIZES: [(usize, usize); 5] = [(160, 90), (60, 40), (240, 135), (240, 40), (60, 135)];

fn state(map: Map, (width, height): (usize, usize), players: usize) -> GameState {
    GameState::new(MatchConfig {
        width,
        height,
        player_count: players,
        map,
        ..MatchConfig::default()
    })
}

#[test]
fn map_names_are_unique() {
    for (i, a) in Map::ALL.iter().enumerate() {
        for b in &Map::ALL[i + 1..] {
            assert_ne!(a.name, b.name);
        }
    }
}

#[test]
fn spawns_are_whole_squares_free_of_obstacles() {
    for map in Map::ALL {
        for size in SIZES {
            for players in 2..=4 {
                let s = state(*map, size, players);
                // A spawn is a square of side min(width, height) / 5.
                let side = size.0.min(size.1) / 5;
                for p in 0..players {
                    assert_eq!(
                        s.fluid_count(p as u8),
                        side * side,
                        "{} at {size:?}: player {p} of {players} lost spawn cells to obstacles",
                        map.name
                    );
                }
            }
        }
    }
}

#[test]
fn no_open_area_is_sealed_off() {
    for map in Map::ALL {
        for size in SIZES {
            let s = state(*map, size, 2);
            let g = &s.grid;
            let (w, h) = (g.width(), g.height());
            let open = |x: usize, y: usize| g.get(x, y) != Cell::Obstacle;
            let total = (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| open(x, y))
                .count();
            // Flood fill (4-connected) from the first open cell.
            let start = (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .find(|&(x, y)| open(x, y))
                .unwrap();
            let mut seen = vec![false; w * h];
            let mut stack = vec![start];
            seen[start.1 * w + start.0] = true;
            let mut reached = 0;
            while let Some((x, y)) = stack.pop() {
                reached += 1;
                let neighbours = [
                    (x.wrapping_sub(1), y),
                    (x + 1, y),
                    (x, y.wrapping_sub(1)),
                    (x, y + 1),
                ];
                for (nx, ny) in neighbours {
                    if nx < w && ny < h && !seen[ny * w + nx] && open(nx, ny) {
                        seen[ny * w + nx] = true;
                        stack.push((nx, ny));
                    }
                }
            }
            assert_eq!(
                reached, total,
                "{} at {size:?} has a sealed-off area",
                map.name
            );
        }
    }
}

#[test]
fn layouts_are_mirror_symmetric() {
    let mirror_x = |r: &Rect| Rect {
        x0: 16 - r.x1,
        x1: 16 - r.x0,
        ..*r
    };
    let mirror_y = |r: &Rect| Rect {
        y0: 16 - r.y1,
        y1: 16 - r.y0,
        ..*r
    };
    for map in Map::ALL {
        for r in map.obstacles {
            assert!(
                map.obstacles.contains(&mirror_x(r)),
                "{}: {r:?} has no left-right mirror",
                map.name
            );
            assert!(
                map.obstacles.contains(&mirror_y(r)),
                "{}: {r:?} has no top-bottom mirror",
                map.name
            );
        }
    }
}

#[test]
fn there_are_several_maps() {
    assert!(
        Map::ALL.len() >= 5,
        "Pillars, Empty and at least three new maps"
    );
}

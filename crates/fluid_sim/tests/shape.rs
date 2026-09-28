//! Without obstacles, fluid settles into a circle around its cursor.

use fluid_sim::{Cell, Command, GameState, MatchConfig, SUBCELL};

/// Share of player 0's fluid cells that lie inside the ideal circle: the disk
/// around the cursor with the same area as the fluid.
fn circle_overlap(s: &GameState) -> f64 {
    let (cx, cy) = s.cursors[0].cell();
    let g = &s.grid;
    let fluid: Vec<(usize, usize)> = (0..g.height())
        .flat_map(|y| (0..g.width()).map(move |x| (x, y)))
        .filter(|&(x, y)| g.get(x, y) == Cell::Fluid(0))
        .collect();
    let radius_sq = fluid.len() as f64 / std::f64::consts::PI;
    let inside = fluid
        .iter()
        .filter(|&&(x, y)| {
            let (dx, dy) = (x as f64 - cx as f64, y as f64 - cy as f64);
            dx * dx + dy * dy <= radius_sq
        })
        .count();
    inside as f64 / fluid.len() as f64
}

#[test]
fn fluid_settles_into_a_circle_around_the_cursor() {
    let config = MatchConfig {
        width: 120,
        height: 90,
        ..MatchConfig::default()
    };
    let mut s = GameState::new(config);
    // Move player 0's cursor to the middle, away from its square blob.
    s.cursors[0].x = 60 * SUBCELL + SUBCELL / 2;
    for _ in 0..400 {
        s.step(&[Command::Idle, Command::Idle]);
    }
    let overlap = circle_overlap(&s);
    // A square or diamond of the same area overlaps the circle by only ~91%.
    assert!(
        overlap >= 0.95,
        "fluid should be round, overlap was {overlap:.3}"
    );
}

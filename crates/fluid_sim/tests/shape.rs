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
    for _ in 0..1200 {
        s.step(&[Command::Idle, Command::Idle]);
    }
    let overlap = circle_overlap(&s);
    // A square or diamond of the same area overlaps the circle by only ~91%.
    assert!(
        overlap >= 0.95,
        "fluid should be round, overlap was {overlap:.3}"
    );
}

/// Share of player 0's fluid cells whose mirror image across the diagonal
/// through the fluid's own centre (swapping x and y offsets) is also player
/// 0's fluid. Measures the shape, not where it is relative to the cursor.
fn diagonal_symmetry(s: &GameState) -> f64 {
    let g = &s.grid;
    let is_fluid = |x: i64, y: i64| {
        (0..g.width() as i64).contains(&x)
            && (0..g.height() as i64).contains(&y)
            && g.get(x as usize, y as usize) == Cell::Fluid(0)
    };
    let fluid: Vec<(i64, i64)> = (0..g.height() as i64)
        .flat_map(|y| (0..g.width() as i64).map(move |x| (x, y)))
        .filter(|&(x, y)| is_fluid(x, y))
        .collect();
    let n = fluid.len() as i64;
    let (cx, cy) = (
        (fluid.iter().map(|c| c.0).sum::<i64>() + n / 2) / n,
        (fluid.iter().map(|c| c.1).sum::<i64>() + n / 2) / n,
    );
    let mirrored = fluid
        .iter()
        .filter(|&&(x, y)| is_fluid(cx + (y - cy), cy + (x - cx)))
        .count();
    mirrored as f64 / fluid.len() as f64
}

#[test]
fn fluid_follows_a_diagonal_cursor_symmetrically() {
    let config = MatchConfig {
        width: 120,
        height: 90,
        ..MatchConfig::default()
    };
    let mut s = GameState::new(config);
    // Player 0's square blob is symmetric about its diagonal; steering along
    // that diagonal should keep the fluid symmetric, not skewed to one side.
    let down_right = [Command::Direction { dx: 1, dy: 1 }, Command::Idle];
    for t in 0..120 {
        s.step(&down_right);
        if t % 30 == 29 {
            let symmetry = diagonal_symmetry(&s);
            assert!(
                symmetry >= 0.9,
                "tick {t}: fluid should stay symmetric, symmetry was {symmetry:.3}"
            );
        }
    }
}

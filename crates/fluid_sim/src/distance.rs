//! Distance fields: how far every cell is from a cursor, going around obstacles.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::{Cell, Grid};

/// Steps in 16 directions with costs 5 (orthogonal), 7 (diagonal) and 11
/// (knight's move): a "chamfer" metric that approximates Euclidean distance to
/// within ~2%, so the cells closest to a cursor form a near-circle, not a diamond.
const MOVES: [(i32, i32, u32); 16] = [
    (1, 0, 5),
    (-1, 0, 5),
    (0, 1, 5),
    (0, -1, 5),
    (1, 1, 7),
    (1, -1, 7),
    (-1, 1, 7),
    (-1, -1, 7),
    (2, 1, 11),
    (2, -1, 11),
    (-2, 1, 11),
    (-2, -1, 11),
    (1, 2, 11),
    (-1, 2, 11),
    (1, -2, 11),
    (-1, -2, 11),
];

/// Distance from `from` to every cell, 5 units per cell. Fluid
/// doesn't block the path, obstacles do. `u32::MAX` means unreachable.
pub fn distance_field(grid: &Grid, from: (usize, usize)) -> Vec<u32> {
    let (w, h) = (grid.width() as i32, grid.height() as i32);
    let cells = grid.cells();
    let open = |x: i32, y: i32| {
        x >= 0 && y >= 0 && x < w && y < h && cells[(y * w + x) as usize] != Cell::Obstacle
    };
    let mut dist = vec![u32::MAX; cells.len()];
    let (sx, sy) = (from.0 as i32, from.1 as i32);
    if !open(sx, sy) {
        return dist;
    }
    let start = (sy * w + sx) as usize;
    dist[start] = 0;
    let mut heap = BinaryHeap::from([Reverse((0u32, start))]);
    while let Some(Reverse((d, i))) = heap.pop() {
        if d > dist[i] {
            continue;
        }
        let (x, y) = (i as i32 % w, i as i32 / w);
        for &(dx, dy, cost) in &MOVES {
            let (nx, ny) = (x + dx, y + dy);
            if !open(nx, ny) || !clears_corners(&open, x, y, dx, dy) {
                continue;
            }
            let n = (ny * w + nx) as usize;
            if d + cost < dist[n] {
                dist[n] = d + cost;
                heap.push(Reverse((d + cost, n)));
            }
        }
    }
    dist
}

/// Diagonal and knight steps may not cut obstacle corners: the cells they
/// pass by must be open too.
fn clears_corners(open: &impl Fn(i32, i32) -> bool, x: i32, y: i32, dx: i32, dy: i32) -> bool {
    match (dx.abs(), dy.abs()) {
        (1, 1) => open(x + dx, y) && open(x, y + dy),
        (2, 1) => open(x + dx.signum(), y) && open(x + dx.signum(), y + dy),
        (1, 2) => open(x, y + dy.signum()) && open(x + dx, y + dy.signum()),
        _ => true,
    }
}

//! Maps: obstacle layouts defined in fractions of the grid, so the same map
//! works at any grid size chosen at setup (ADR 0003).

use crate::{Cell, Grid};

/// A rectangle in sixteenths of the grid's width and height, covering
/// `[x0, x1) × [y0, y1)`. `x1 = 16` reaches the right edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x0: u8,
    pub y0: u8,
    pub x1: u8,
    pub y1: u8,
}

/// A named obstacle layout. Built-in maps keep the spawn areas free and are
/// symmetric, so every player starts with the same amount of fluid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Map {
    pub name: &'static str,
    pub obstacles: &'static [Rect],
}

const fn rect(x0: u8, y0: u8, x1: u8, y1: u8) -> Rect {
    Rect { x0, y0, x1, y1 }
}

impl Map {
    /// No obstacles.
    pub const EMPTY: Map = Map {
        name: "Empty",
        obstacles: &[],
    };

    /// Four pillars around the centre and a short wall near each corner.
    pub const PILLARS: Map = Map {
        name: "Pillars",
        obstacles: &[
            rect(6, 6, 7, 7),
            rect(9, 6, 10, 7),
            rect(6, 9, 7, 10),
            rect(9, 9, 10, 10),
            rect(1, 3, 2, 6),
            rect(14, 3, 15, 6),
            rect(1, 10, 2, 13),
            rect(14, 10, 15, 13),
        ],
    };

    /// Mark this map's obstacles on `grid`, scaled to its size.
    pub(crate) fn apply(&self, grid: &mut Grid) {
        let (w, h) = (grid.width(), grid.height());
        for r in self.obstacles {
            let scale = |v: u8, size: usize| usize::from(v) * size / 16;
            for y in scale(r.y0, h)..scale(r.y1, h) {
                for x in scale(r.x0, w)..scale(r.x1, w) {
                    grid.set(x, y, Cell::Obstacle);
                }
            }
        }
    }
}

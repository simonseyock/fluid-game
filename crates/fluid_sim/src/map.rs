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

    /// Long walls split the map into lanes, with a block in the centre:
    /// getting at an opponent means flanking.
    pub const CORRIDORS: Map = Map {
        name: "Corridors",
        obstacles: &[
            rect(0, 4, 6, 5),
            rect(10, 4, 16, 5),
            rect(0, 11, 6, 12),
            rect(10, 11, 16, 12),
            rect(7, 7, 9, 9),
        ],
    };

    /// An open centre inside a ring of four big pillars.
    pub const ARENA: Map = Map {
        name: "Arena",
        obstacles: &[
            rect(4, 4, 6, 6),
            rect(10, 4, 12, 6),
            rect(4, 10, 6, 12),
            rect(10, 10, 12, 12),
        ],
    };

    /// Diagonal walls fence each spawn into its own zone. Every zone opens
    /// wide onto the central hub and through a narrow slip at its corner.
    pub const DIAGONALS: Map = Map {
        name: "Diagonals",
        obstacles: &[
            rect(1, 1, 2, 2),
            rect(2, 2, 3, 3),
            rect(3, 3, 4, 4),
            rect(4, 4, 5, 5),
            rect(14, 1, 15, 2),
            rect(13, 2, 14, 3),
            rect(12, 3, 13, 4),
            rect(11, 4, 12, 5),
            rect(1, 14, 2, 15),
            rect(2, 13, 3, 14),
            rect(3, 12, 4, 13),
            rect(4, 11, 5, 12),
            rect(14, 14, 15, 15),
            rect(13, 13, 14, 14),
            rect(12, 12, 13, 13),
            rect(11, 11, 12, 12),
        ],
    };

    /// A small labyrinth: bent walls around every spawn, short stubs at the
    /// edges and bars across the centre.
    pub const LABYRINTH: Map = Map {
        name: "Labyrinth",
        obstacles: &[
            rect(0, 6, 1, 7),
            rect(0, 9, 1, 10),
            rect(1, 1, 5, 2),
            rect(1, 2, 2, 5),
            rect(1, 11, 2, 14),
            rect(1, 14, 5, 15),
            rect(3, 3, 5, 4),
            rect(3, 4, 4, 6),
            rect(3, 10, 4, 12),
            rect(3, 12, 5, 13),
            rect(5, 0, 6, 1),
            rect(5, 15, 6, 16),
            rect(7, 6, 9, 7),
            rect(7, 9, 9, 10),
            rect(10, 0, 11, 1),
            rect(10, 15, 11, 16),
            rect(11, 1, 15, 2),
            rect(11, 3, 13, 4),
            rect(11, 12, 13, 13),
            rect(11, 14, 15, 15),
            rect(12, 4, 13, 6),
            rect(12, 10, 13, 12),
            rect(14, 2, 15, 5),
            rect(14, 11, 15, 14),
            rect(15, 6, 16, 7),
            rect(15, 9, 16, 10),
        ],
    };

    /// One huge block in the middle, filling the space between the spawns.
    pub const MONOLITH: Map = Map {
        name: "Monolith",
        obstacles: &[rect(6, 6, 10, 10)],
    };

    /// Every colour starts in a U-shaped bunker: the central block behind it,
    /// walls on both sides, open only toward the map edge. The fight begins
    /// by breaking out and going round.
    pub const BUNKERS: Map = Map {
        name: "Bunkers",
        obstacles: &[
            rect(6, 6, 10, 10),
            rect(3, 5, 6, 6),
            rect(10, 5, 13, 6),
            rect(3, 10, 6, 11),
            rect(10, 10, 13, 11),
            rect(5, 3, 6, 5),
            rect(10, 3, 11, 5),
            rect(5, 11, 6, 13),
            rect(10, 11, 11, 13),
        ],
    };

    /// Every built-in map, in the order the setup screen lists them.
    pub const ALL: &'static [Map] = &[
        Map::PILLARS,
        Map::ARENA,
        Map::CORRIDORS,
        Map::DIAGONALS,
        Map::LABYRINTH,
        Map::MONOLITH,
        Map::BUNKERS,
        Map::EMPTY,
    ];

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

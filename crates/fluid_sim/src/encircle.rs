//! Encirclement, prototype rule (issue #5; the final rules get their own ADR).
//!
//! A local majority vote: a fluid cell flips to an opponent's colour when more
//! of its 8 neighbours are opponents than its own colour. Flat fronts hold,
//! bulges into enemy fluid are lost, and small enclosed blobs are eaten from
//! the outside in.

use crate::state::MAX_PLAYERS as MAX_PLAYER_SLOTS;
use crate::{Cell, Grid};

impl Grid {
    /// Apply one round of the majority rule to every fluid cell at once, all
    /// deciding from the same snapshot. A cell flips to the opponent with the
    /// most neighbours; a tie between opponents means no flip. Empty cells and
    /// obstacles don't count. Returns how many cells flipped.
    pub fn flip_outnumbered(&mut self) -> usize {
        let (w, h) = (self.width() as i64, self.height() as i64);
        let snapshot = self.cells().to_vec();
        let mut flips = Vec::new();
        for (i, &cell) in snapshot.iter().enumerate() {
            let Cell::Fluid(own) = cell else { continue };
            let (x, y) = (i as i64 % w, i as i64 / w);
            let mut counts = [0u8; MAX_PLAYER_SLOTS];
            for (dx, dy) in NEIGHBOURS {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                if let Cell::Fluid(p) = snapshot[(ny * w + nx) as usize] {
                    counts[p as usize] += 1;
                }
            }
            let own_count = counts[own as usize];
            let opponents: u8 = counts.iter().sum::<u8>() - own_count;
            if opponents <= own_count {
                continue;
            }
            let best = (0..MAX_PLAYER_SLOTS)
                .filter(|&p| p != own as usize)
                .max_by_key(|&p| counts[p])
                .expect("there is at least one opponent slot");
            let tied = (0..MAX_PLAYER_SLOTS)
                .any(|p| p != own as usize && p != best && counts[p] == counts[best]);
            if !tied {
                flips.push((i, Cell::Fluid(best as u8)));
            }
        }
        let cells = self.cells_mut();
        for &(i, cell) in &flips {
            cells[i] = cell;
        }
        flips.len()
    }
}

const NEIGHBOURS: [(i64, i64); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

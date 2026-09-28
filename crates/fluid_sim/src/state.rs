use crate::distance::distance_field;
use crate::{Cell, Command, Grid, Map, PlayerId};

/// Cursor positions are fixed-point: one cell is `SUBCELL` units wide. This lets
/// cursors move slower than one cell per tick without floats (ADR 0002, 0006).
pub const SUBCELL: i32 = 256;

/// Most player slots a match can have (ADR 0005).
pub const MAX_PLAYERS: usize = 4;

/// Spawn anchors as fractions (numerator / 8) of grid width and height, per slot.
const SPAWN_ANCHORS: [(usize, usize); MAX_PLAYERS] = [(2, 4), (6, 4), (4, 2), (4, 6)];

/// Pressure moves at most 1 in `PRESSURE_RATE` of a blob's edge cells per tick,
/// so reshaping looks like flowing rather than jumping.
const PRESSURE_RATE: usize = 8;

/// Share of fluid steps (out of 256) that allow diagonal moves: (√2 − 1)·256.
const DIAGONAL_SHARE: u64 = 106;

/// Settings chosen before a match (ADR 0003, 0005).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchConfig {
    pub width: usize,
    pub height: usize,
    pub player_count: usize,
    /// Max cursor movement per tick in any direction, in `SUBCELL` units.
    /// The same for every player and input device (ADR 0006).
    pub cursor_speed: i32,
    /// Fluid steps per tick, in `SUBCELL` units (`SUBCELL` = one step every
    /// tick). Lower than `cursor_speed`, so fluid trails behind its cursor.
    pub fluid_speed: i32,
    /// Obstacle layout, scaled to the grid size.
    pub map: Map,
    /// Match length in ticks. The simulation only counts ticks; the app
    /// decides how many ticks make a second (ADR 0002).
    pub match_ticks: u64,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            width: 160,
            height: 90,
            player_count: 2,
            cursor_speed: SUBCELL / 2,
            fluid_speed: SUBCELL / 3,
            map: Map::EMPTY,
            // Three minutes at 60 ticks per second.
            match_ticks: 3 * 60 * 60,
        }
    }
}

/// A player's cursor, owned by the simulation (ADR 0006).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    /// Position in `SUBCELL` units.
    pub x: i32,
    pub y: i32,
}

impl Cursor {
    /// The grid cell the cursor is in.
    pub fn cell(&self) -> (usize, usize) {
        ((self.x / SUBCELL) as usize, (self.y / SUBCELL) as usize)
    }
}

/// How a match ended (ADR 0005).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The last colour left, or the most cells when time ran out.
    Winner(PlayerId),
    /// Time ran out with these players sharing the most cells.
    Tie(Vec<PlayerId>),
}

/// The whole match state. Two states built from the same config and fed the
/// same commands stay equal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub config: MatchConfig,
    pub grid: Grid,
    pub cursors: Vec<Cursor>,
    pub tick: u64,
}

impl GameState {
    /// A fresh match: each player gets a square blob of fluid with their
    /// cursor at its centre.
    pub fn new(config: MatchConfig) -> Self {
        assert!(
            (2..=MAX_PLAYERS).contains(&config.player_count),
            "player_count must be 2..={MAX_PLAYERS}"
        );
        let (w, h) = (config.width, config.height);
        let mut grid = Grid::new(w, h);
        config.map.apply(&mut grid);
        let side = (w.min(h) / 5).max(1);
        let mut cursors = Vec::with_capacity(config.player_count);
        for (p, &(ax, ay)) in SPAWN_ANCHORS.iter().take(config.player_count).enumerate() {
            let (cx, cy) = (w * ax / 8, h * ay / 8);
            let (x0, y0) = (cx.saturating_sub(side / 2), cy.saturating_sub(side / 2));
            for y in y0..(y0 + side).min(h) {
                for x in x0..(x0 + side).min(w) {
                    if grid.get(x, y) == Cell::Empty {
                        grid.set(x, y, Cell::Fluid(p as PlayerId));
                    }
                }
            }
            cursors.push(Cursor {
                x: cx as i32 * SUBCELL + SUBCELL / 2,
                y: cy as i32 * SUBCELL + SUBCELL / 2,
            });
        }
        Self {
            config,
            grid,
            cursors,
            tick: 0,
        }
    }

    pub fn player_count(&self) -> usize {
        self.cursors.len()
    }

    /// Number of fluid cells a player owns.
    pub fn fluid_count(&self, player: PlayerId) -> usize {
        self.grid.count(Cell::Fluid(player))
    }

    /// Ticks left on the match clock.
    pub fn remaining_ticks(&self) -> u64 {
        self.config.match_ticks.saturating_sub(self.tick)
    }

    /// A player whose colour has no cells left is out (ADR 0005).
    pub fn is_eliminated(&self, player: PlayerId) -> bool {
        self.fluid_count(player) == 0
    }

    /// `None` while the match is running. Otherwise the last colour left, or,
    /// once the clock runs out, the player with the most cells (ties stay ties).
    pub fn outcome(&self) -> Option<Outcome> {
        let counts: Vec<usize> = (0..self.player_count() as PlayerId)
            .map(|p| self.fluid_count(p))
            .collect();
        let alive: Vec<PlayerId> = (0..counts.len() as PlayerId)
            .filter(|&p| counts[p as usize] > 0)
            .collect();
        if let [last] = alive[..] {
            return Some(Outcome::Winner(last));
        }
        if self.remaining_ticks() > 0 {
            return None;
        }
        let most = counts.iter().copied().max().unwrap_or(0);
        let leaders: Vec<PlayerId> = (0..counts.len() as PlayerId)
            .filter(|&p| counts[p as usize] == most)
            .collect();
        Some(match leaders[..] {
            [winner] => Outcome::Winner(winner),
            _ => Outcome::Tie(leaders),
        })
    }

    /// Advance one tick. `commands[p]` is player `p`'s command; missing
    /// entries count as `Idle`. Does nothing once the match is over.
    pub fn step(&mut self, commands: &[Command]) {
        if self.outcome().is_some() {
            return;
        }
        for p in 0..self.player_count() {
            let command = commands.get(p).copied().unwrap_or_default();
            self.move_cursor(p, command);
        }
        // Fluid steps on a share of ticks set by `fluid_speed`, spread evenly.
        let speed = self.config.fluid_speed.max(0) as u64;
        let sub = SUBCELL as u64;
        let first_step = (self.tick * speed) / sub;
        let fluid_steps = ((self.tick + 1) * speed) / sub - first_step;
        for n in first_step..first_step + fluid_steps {
            let distances: Vec<Vec<u32>> = (0..self.player_count())
                .map(|p| distance_field(&self.grid, self.cursors[p].cell()))
                .collect();
            // A diagonal step covers √2 cells. Allowing them on only a share
            // of steps (√2 − 1 ≈ 41%, orthogonal steps otherwise) makes fluid
            // as fast along diagonals as along axes.
            let diagonal = (n + 1) * DIAGONAL_SHARE / 256 > n * DIAGONAL_SHARE / 256;
            self.move_fluid(&distances, diagonal);
            for (p, dist) in distances.iter().enumerate() {
                self.equalize_pressure(p as PlayerId, dist);
            }
            // Fluid pushes into a neighbour that is closer to its own cursor.
            let cells = self.grid.cells().to_vec();
            self.grid.flip_outnumbered(|from, into| match cells[from] {
                Cell::Fluid(p) => {
                    let dist = &distances[p as usize];
                    dist[into] < dist[from]
                }
                _ => false,
            });
        }
        self.tick += 1;
    }

    fn move_cursor(&mut self, p: usize, command: Command) {
        let speed = self.config.cursor_speed;
        let cursor = &mut self.cursors[p];
        let (dx, dy) = match command {
            Command::Idle => (0, 0),
            Command::Direction { dx, dy } => (
                i64::from(dx.signum()) * i64::from(speed),
                i64::from(dy.signum()) * i64::from(speed),
            ),
            Command::Target { x, y } => {
                let tx = i32::from(x) * SUBCELL + SUBCELL / 2;
                let ty = i32::from(y) * SUBCELL + SUBCELL / 2;
                (i64::from(tx - cursor.x), i64::from(ty - cursor.y))
            }
        };
        // Cap the length of the move, not each axis, so diagonals aren't faster.
        let length = (dx * dx + dy * dy).isqrt();
        let (dx, dy) = if length > i64::from(speed) {
            (
                dx * i64::from(speed) / length,
                dy * i64::from(speed) / length,
            )
        } else {
            (dx, dy)
        };
        let (dx, dy) = (dx as i32, dy as i32);
        let max_x = self.config.width as i32 * SUBCELL - 1;
        let max_y = self.config.height as i32 * SUBCELL - 1;
        cursor.x = (cursor.x + dx).clamp(0, max_x);
        cursor.y = (cursor.y + dy).clamp(0, max_y);
    }

    /// Each fluid cell moves at most one step toward its own cursor, into the
    /// closest empty neighbour (including diagonals). Cells closest to their
    /// cursor move first, so a blob flows as a whole instead of stretching out.
    fn move_fluid(&mut self, distances: &[Vec<u32>], diagonal: bool) {
        let players = self.player_count() as u64;
        let rotation = (self.tick % players) as usize;
        let mut order: Vec<(u32, usize, usize)> = self
            .grid
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, cell)| match *cell {
                Cell::Fluid(p) => {
                    let p = p as usize;
                    // Rotate which player wins ties so no slot is always first.
                    let priority = (p + self.player_count() - rotation) % self.player_count();
                    Some((distances[p][i], priority, i))
                }
                _ => None,
            })
            .collect();
        order.sort_unstable();

        let mut moved = vec![false; self.grid.cells().len()];
        for (d, _, i) in order {
            if d == 0 || d == u32::MAX || moved[i] {
                continue;
            }
            let Cell::Fluid(p) = self.grid.cells()[i] else {
                continue;
            };
            let dist = &distances[p as usize];
            let cells = self.grid.cells();
            // Diagonal steps let fluid flow straight along diagonals instead of
            // staircasing, but never squeeze past obstacles or other fluid.
            let passable = |c: Cell| c == Cell::Empty || c == Cell::Fluid(p);
            let diagonals = self
                .grid
                .diagonals(i)
                .filter(|&(_, a, b)| diagonal && passable(cells[a]) && passable(cells[b]))
                .map(|(n, _, _)| n);
            let target = self
                .grid
                .neighbours(i)
                .chain(diagonals)
                .filter(|&n| cells[n] == Cell::Empty && dist[n] < d)
                .min_by_key(|&n| dist[n]);
            if let Some(n) = target {
                let cells = self.grid.cells_mut();
                cells[n] = Cell::Fluid(p);
                cells[i] = Cell::Empty;
                moved[n] = true;
            }
        }
    }

    /// Pressure: within each connected blob of player `p`, fluid drains from
    /// the edge cells furthest from the blob's centre into the free cells next
    /// to the blob that are closest to it, like a droplet pulling itself round.
    /// Measuring from the blob's own centre (not the cursor) means pressure
    /// reshapes a blob without carrying it along: travel comes from
    /// `move_fluid`, capped by `fluid_speed`. At rest the blob drifts onto its
    /// cursor and settles into a circle around it.
    fn equalize_pressure(&mut self, p: PlayerId, dist: &[u32]) {
        let fluid = Cell::Fluid(p);
        let cursor = self.cursors[p as usize].cell();
        let cursor_index = self.grid.index(cursor.0, cursor.1);
        let width = self.grid.width() as i64;
        let mut seen = vec![false; self.grid.cells().len()];
        for start in 0..seen.len() {
            if seen[start] || self.grid.cells()[start] != fluid {
                continue;
            }
            // Collect one 4-connected blob, its edge cells and the free cells around it.
            let (mut edge, mut free) = (Vec::new(), Vec::new());
            let (mut count, mut sum_x, mut sum_y) = (0i64, 0i64, 0i64);
            let mut holds_cursor = false;
            let mut stack = vec![start];
            seen[start] = true;
            while let Some(i) = stack.pop() {
                holds_cursor |= i == cursor_index;
                count += 1;
                sum_x += i as i64 % width;
                sum_y += i as i64 / width;
                let mut on_edge = false;
                for n in self.grid.neighbours(i) {
                    match self.grid.cells()[n] {
                        c if c == fluid && !seen[n] => {
                            seen[n] = true;
                            stack.push(n);
                        }
                        Cell::Empty => {
                            on_edge = true;
                            if dist[n] != u32::MAX {
                                free.push(n);
                            }
                        }
                        _ => {}
                    }
                }
                if on_edge {
                    edge.push(i);
                }
            }
            // Pressure centres on an anchor: the centre of mass, or, once the
            // cursor is inside this blob, the centre nudged one cell per axis
            // toward the cursor so the blob drifts onto it. While the cursor
            // is ahead of the blob there's no nudge, so pressure never carries
            // fluid faster than it flows. Scaled by `count` to stay in integers.
            let (kx, ky) = (cursor.0 as i64 * count, cursor.1 as i64 * count);
            let nudge = if holds_cursor { count } else { 0 };
            let anchor_x = sum_x + (kx - sum_x).clamp(-nudge, nudge);
            let anchor_y = sum_y + (ky - sum_y).clamp(-nudge, nudge);
            let potential = |i: usize| {
                let (x, y) = (i as i64 % width, i as i64 / width);
                let (dx, dy) = (x * count - anchor_x, y * count - anchor_y);
                dx * dx + dy * dy
            };
            free.sort_unstable_by_key(|&n| (potential(n), n));
            free.dedup();
            edge.sort_unstable_by_key(|&i| (std::cmp::Reverse(potential(i)), i));
            let budget = edge.len().div_ceil(PRESSURE_RATE);
            let cells = self.grid.cells_mut();
            for (&from, &to) in edge.iter().zip(&free).take(budget) {
                if potential(to) >= potential(from) {
                    break;
                }
                cells[to] = fluid;
                cells[from] = Cell::Empty;
            }
        }
    }
}

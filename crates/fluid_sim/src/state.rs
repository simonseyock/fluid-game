use crate::distance::distance_field;
use crate::{Cell, Command, Grid, PlayerId};

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

/// Settings chosen before a match (ADR 0003, 0005).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchConfig {
    pub width: usize,
    pub height: usize,
    pub player_count: usize,
    /// Max cursor movement per tick along each axis, in `SUBCELL` units.
    /// The same for every player and input device (ADR 0006).
    pub cursor_speed: i32,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            width: 160,
            height: 90,
            player_count: 2,
            cursor_speed: SUBCELL / 2,
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
        let side = (w.min(h) / 5).max(1);
        let mut cursors = Vec::with_capacity(config.player_count);
        for (p, &(ax, ay)) in SPAWN_ANCHORS.iter().take(config.player_count).enumerate() {
            let (cx, cy) = (w * ax / 8, h * ay / 8);
            let (x0, y0) = (cx.saturating_sub(side / 2), cy.saturating_sub(side / 2));
            for y in y0..(y0 + side).min(h) {
                for x in x0..(x0 + side).min(w) {
                    grid.set(x, y, Cell::Fluid(p as PlayerId));
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

    /// Advance one tick. `commands[p]` is player `p`'s command; missing
    /// entries count as `Idle`.
    pub fn step(&mut self, commands: &[Command]) {
        for p in 0..self.player_count() {
            let command = commands.get(p).copied().unwrap_or_default();
            self.move_cursor(p, command);
        }
        let distances: Vec<Vec<u32>> = (0..self.player_count())
            .map(|p| distance_field(&self.grid, self.cursors[p].cell()))
            .collect();
        self.move_fluid(&distances);
        for (p, dist) in distances.iter().enumerate() {
            self.equalize_pressure(p as PlayerId, dist);
        }
        self.tick += 1;
    }

    fn move_cursor(&mut self, p: usize, command: Command) {
        let speed = self.config.cursor_speed;
        let cursor = &mut self.cursors[p];
        let (dx, dy) = match command {
            Command::Idle => (0, 0),
            Command::Direction { dx, dy } => (
                i32::from(dx.signum()) * speed,
                i32::from(dy.signum()) * speed,
            ),
            Command::Target { x, y } => {
                let tx = i32::from(x) * SUBCELL + SUBCELL / 2;
                let ty = i32::from(y) * SUBCELL + SUBCELL / 2;
                (
                    (tx - cursor.x).clamp(-speed, speed),
                    (ty - cursor.y).clamp(-speed, speed),
                )
            }
        };
        let max_x = self.config.width as i32 * SUBCELL - 1;
        let max_y = self.config.height as i32 * SUBCELL - 1;
        cursor.x = (cursor.x + dx).clamp(0, max_x);
        cursor.y = (cursor.y + dy).clamp(0, max_y);
    }

    /// Each fluid cell moves at most one step toward its own cursor, into the
    /// closest empty neighbour. Cells closest to their cursor move first, so a
    /// blob flows as a whole instead of stretching out.
    fn move_fluid(&mut self, distances: &[Vec<u32>]) {
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
            let target = self
                .grid
                .neighbours(i)
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
    /// the edge cells furthest from the cursor into the free cells next to the
    /// blob that are closest to it, like water levelling out. Only moves that
    /// bring fluid closer happen, so the blob settles into the shape closest
    /// to its cursor: a circle when nothing is in the way.
    fn equalize_pressure(&mut self, p: PlayerId, dist: &[u32]) {
        let fluid = Cell::Fluid(p);
        let mut seen = vec![false; self.grid.cells().len()];
        for start in 0..seen.len() {
            if seen[start] || self.grid.cells()[start] != fluid {
                continue;
            }
            // Collect one 4-connected blob, its edge cells and the free cells around it.
            let (mut edge, mut free) = (Vec::new(), Vec::new());
            let mut stack = vec![start];
            seen[start] = true;
            while let Some(i) = stack.pop() {
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
            free.sort_unstable_by_key(|&n| (dist[n], n));
            free.dedup();
            edge.sort_unstable_by_key(|&i| (std::cmp::Reverse(dist[i]), i));
            let budget = edge.len().div_ceil(PRESSURE_RATE);
            let cells = self.grid.cells_mut();
            for (&from, &to) in edge.iter().zip(&free).take(budget) {
                if dist[to] >= dist[from] {
                    break;
                }
                cells[to] = fluid;
                cells[from] = Cell::Empty;
            }
        }
    }
}

/// Index of a player slot, `0..player_count` (ADR 0005).
pub type PlayerId = u8;

/// One grid cell (ADR 0003).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cell {
    #[default]
    Empty,
    Obstacle,
    Fluid(PlayerId),
}

/// Row-major grid of cells. Its size is chosen at setup and fixed for the match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0, "grid must not be empty");
        Self {
            width,
            height,
            cells: vec![Cell::Empty; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width && y < self.height);
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.index(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        let i = self.index(x, y);
        self.cells[i] = cell;
    }

    pub fn count(&self, cell: Cell) -> usize {
        self.cells.iter().filter(|&&c| c == cell).count()
    }

    pub(crate) fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }

    /// The 4-connected neighbours of cell index `i` that lie inside the grid,
    /// in the fixed order up, right, down, left.
    pub(crate) fn neighbours(&self, i: usize) -> impl Iterator<Item = usize> + use<> {
        let (w, h) = (self.width, self.height);
        let (x, y) = (i % w, i / w);
        [
            (y > 0).then(|| i - w),
            (x + 1 < w).then(|| i + 1),
            (y + 1 < h).then(|| i + w),
            (x > 0).then(|| i - 1),
        ]
        .into_iter()
        .flatten()
    }
}

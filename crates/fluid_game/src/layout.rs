//! Where the grid sits in the window (ADR 0003): scaled to the largest size
//! that fits, cells kept square, centred with margins. Shared by the renderer
//! and the mouse input, so drawing and pointing always agree.

use bevy::math::Vec2;

/// The grid's placement in window coordinates (logical pixels, origin at the
/// window's top-left, y pointing down, as `Window::cursor_position` reports).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridLayout {
    /// Side length of one cell.
    pub cell: f32,
    /// Top-left corner of the grid.
    pub origin: Vec2,
    grid: (usize, usize),
}

impl GridLayout {
    /// Fit a `grid` of (width, height) cells into a window of `window` size.
    pub fn fit(window: Vec2, grid: (usize, usize)) -> Self {
        let cells = Vec2::new(grid.0 as f32, grid.1 as f32);
        let cell = (window.x / cells.x).min(window.y / cells.y);
        let origin = (window - cells * cell) / 2.0;
        Self { cell, origin, grid }
    }

    /// The grid's size in window pixels.
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.grid.0 as f32, self.grid.1 as f32) * self.cell
    }

    /// The cell under a window position, clamped to the grid's edge when the
    /// position is in a margin or outside the window.
    pub fn cell_at(&self, position: Vec2) -> (u16, u16) {
        let local = ((position - self.origin) / self.cell).floor();
        let clamp = |v: f32, len: usize| v.clamp(0.0, (len - 1) as f32) as u16;
        (clamp(local.x, self.grid.0), clamp(local.y, self.grid.1))
    }
}

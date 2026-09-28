//! Draws the grid as one texture, one texel per cell, scaled to fit the window
//! (ADR 0003). Neon colors on black, with HDR bloom so the fluid glows.

use bevy::asset::RenderAssetUsages;
use bevy::camera::Hdr;
use bevy::image::ImageSampler;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use fluid_sim::{Cell, GameState};

use crate::Sim;
use crate::layout::GridLayout;

/// Neon color per player slot, as RGBA.
pub const PLAYER_COLORS: [[u8; 4]; 4] = [
    [0, 255, 255, 255], // cyan
    [255, 0, 200, 255], // magenta
    [170, 255, 0, 255], // lime
    [255, 140, 0, 255], // orange
];
/// Display name per player slot, matching `PLAYER_COLORS`.
pub const PLAYER_NAMES: [&str; 4] = ["Cyan", "Magenta", "Lime", "Orange"];
/// Brightness boost for the grid in HDR. Colours above 1.0 feed the bloom,
/// so bright fluid glows while dim obstacles and black background stay put.
const GLOW: f32 = 2.5;
const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const OBSTACLE: [u8; 4] = [14, 14, 32, 255];
/// A cursor: a plus-shaped cross, and the cells around it as its outline.
const CURSOR_CROSS: [(i64, i64); 5] = [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)];
const CURSOR_OUTLINE: [(i64, i64); 8] = [
    (-2, 0),
    (2, 0),
    (0, -2),
    (0, 2),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
];

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup, setup)
            .add_systems(Update, (draw_grid, fit_to_window));
    }
}

#[derive(Component)]
struct GridSprite;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, sim: Res<Sim>) {
    let grid = &sim.0.grid;
    let size = Extent3d {
        width: grid.width() as u32,
        height: grid.height() as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &BACKGROUND,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    // Crisp cells, no smoothing between them.
    image.sampler = ImageSampler::nearest();
    commands.spawn((Camera2d, Hdr, Bloom::NATURAL));
    commands.spawn((
        Sprite {
            color: Color::linear_rgb(GLOW, GLOW, GLOW),
            ..Sprite::from_image(images.add(image))
        },
        GridSprite,
    ));
}

fn draw_grid(
    sim: Res<Sim>,
    sprite: Single<&Sprite, With<GridSprite>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut image) = images.get_mut(&sprite.image) else {
        return;
    };
    // The grid size can change between matches (setup screen): keep one
    // texel per cell.
    let grid = &sim.0.grid;
    let (w, h) = (grid.width() as u32, grid.height() as u32);
    if (image.width(), image.height()) != (w, h) {
        image.resize(Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        });
    }
    if let Some(data) = image.data.as_mut() {
        paint(&sim.0, data);
    }
}

/// Paint `state` into an RGBA buffer with one texel per cell, row by row.
/// Cursors go on top: a cross in the player's colour inside a black outline,
/// so it stands out on black background and on any fluid, its own included.
pub fn paint(state: &GameState, data: &mut [u8]) {
    for (texel, cell) in data
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(state.grid.cells())
    {
        let color = match *cell {
            Cell::Empty => BACKGROUND,
            Cell::Obstacle => OBSTACLE,
            Cell::Fluid(p) => player_color(p as usize),
        };
        texel.copy_from_slice(&color);
    }
    let (w, h) = (state.grid.width() as i64, state.grid.height() as i64);
    let mut put = |x: i64, y: i64, color: &[u8; 4]| {
        if (0..w).contains(&x) && (0..h).contains(&y) {
            let i = (y * w + x) as usize * 4;
            data[i..i + 4].copy_from_slice(color);
        }
    };
    for (p, cursor) in state.cursors.iter().enumerate() {
        let (cx, cy) = cursor.cell();
        let (cx, cy) = (cx as i64, cy as i64);
        for (dx, dy) in CURSOR_OUTLINE {
            put(cx + dx, cy + dy, &BACKGROUND);
        }
        for (dx, dy) in CURSOR_CROSS {
            put(cx + dx, cy + dy, &player_color(p));
        }
    }
}

fn player_color(p: usize) -> [u8; 4] {
    PLAYER_COLORS[p % PLAYER_COLORS.len()]
}

/// Scale the grid to the largest size that fits the window, keeping cells square.
fn fit_to_window(
    window: Single<&Window>,
    sim: Res<Sim>,
    mut sprite: Single<&mut Sprite, With<GridSprite>>,
) {
    let grid = (sim.0.grid.width(), sim.0.grid.height());
    sprite.custom_size = Some(GridLayout::fit(window.size(), grid).size());
}

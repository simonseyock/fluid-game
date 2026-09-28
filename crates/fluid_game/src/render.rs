//! Draws the grid as one texture, one texel per cell, scaled to fit the window
//! (ADR 0003). Neon colors on black.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use fluid_sim::Cell;

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
const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const OBSTACLE: [u8; 4] = [40, 40, 70, 255];
const CURSOR: [u8; 4] = [255, 255, 255, 255];

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
    commands.spawn(Camera2d);
    commands.spawn((Sprite::from_image(images.add(image)), GridSprite));
}

fn draw_grid(
    sim: Res<Sim>,
    sprite: Single<&Sprite, With<GridSprite>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut image) = images.get_mut(&sprite.image) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let state = &sim.0;
    for (texel, cell) in data
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(state.grid.cells())
    {
        let color = match *cell {
            Cell::Empty => BACKGROUND,
            Cell::Obstacle => OBSTACLE,
            Cell::Fluid(p) => PLAYER_COLORS[p as usize % PLAYER_COLORS.len()],
        };
        texel.copy_from_slice(&color);
    }
    // Cursors as small white crosses.
    let (w, h) = (state.grid.width(), state.grid.height());
    for cursor in &state.cursors {
        let (cx, cy) = cursor.cell();
        for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (x, y) = (cx as i64 + dx, cy as i64 + dy);
            if (0..w as i64).contains(&x) && (0..h as i64).contains(&y) {
                let i = (y as usize * w + x as usize) * 4;
                data[i..i + 4].copy_from_slice(&CURSOR);
            }
        }
    }
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

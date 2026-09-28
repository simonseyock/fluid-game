//! Local input: grid layout in the window, pointer-to-cell conversion, and
//! per-slot devices (ADR 0004, 0005, 0006).

use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::KeyCode;
use fluid_game::input::{Device, commands};
use fluid_game::layout::GridLayout;
use fluid_sim::Command;

const GRID: (usize, usize) = (160, 90);

#[test]
fn grid_fills_a_window_of_the_same_shape() {
    let layout = GridLayout::fit(Vec2::new(1280.0, 720.0), GRID);
    assert_eq!(layout.cell, 8.0);
    assert_eq!(layout.origin, Vec2::ZERO);
    assert_eq!(layout.size(), Vec2::new(1280.0, 720.0));
}

#[test]
fn grid_is_centred_with_margins_in_a_narrower_window() {
    let layout = GridLayout::fit(Vec2::new(800.0, 720.0), GRID);
    assert_eq!(layout.cell, 5.0);
    // 450 px tall grid in a 720 px window: 135 px above and below.
    assert_eq!(layout.origin, Vec2::new(0.0, 135.0));
}

#[test]
fn pointer_position_maps_to_the_cell_under_it() {
    let layout = GridLayout::fit(Vec2::new(800.0, 720.0), GRID);
    assert_eq!(layout.cell_at(Vec2::new(0.0, 135.0)), (0, 0));
    assert_eq!(layout.cell_at(Vec2::new(402.0, 360.0)), (80, 45));
    assert_eq!(layout.cell_at(Vec2::new(799.0, 584.0)), (159, 89));
}

#[test]
fn pointer_outside_the_grid_clamps_to_its_edge() {
    let layout = GridLayout::fit(Vec2::new(800.0, 720.0), GRID);
    // In the margin above the grid, and past the right edge.
    assert_eq!(layout.cell_at(Vec2::new(-50.0, 10.0)), (0, 0));
    assert_eq!(layout.cell_at(Vec2::new(900.0, 700.0)), (159, 89));
}

#[test]
fn each_slot_reads_its_own_device() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyD);
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowLeft);
    let devices = [Device::KeyZone(0), Device::KeyZone(1), Device::Mouse];
    assert_eq!(
        commands(&devices, &keys, Some((12, 34))),
        vec![
            Command::Direction { dx: 1, dy: 0 },
            Command::Direction { dx: -1, dy: -1 },
            Command::Target { x: 12, y: 34 },
        ]
    );
}

#[test]
fn idle_without_input() {
    let keys = ButtonInput::<KeyCode>::default();
    let devices = [Device::KeyZone(0), Device::Mouse];
    // No keys held, and the mouse has never been over the grid.
    assert_eq!(
        commands(&devices, &keys, None),
        vec![Command::Idle, Command::Idle]
    );
}

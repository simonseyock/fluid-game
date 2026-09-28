//! Local input sources (ADR 0004, 0005, 0006): each player slot on this
//! device is driven by one device. Key zones on the shared keyboard produce
//! `Direction` commands; the mouse produces `Target` commands in grid cells.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use fluid_sim::Command;

use crate::Sim;
use crate::layout::GridLayout;

/// The device driving one local player slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    /// A key zone on the shared keyboard, by index into `ZONES`.
    KeyZone(usize),
    /// The mouse. The OS merges all mice into one pointer, so at most one
    /// slot can use it (ADR 0004).
    Mouse,
}

/// Devices for the local player slots, in slot order.
#[derive(Resource)]
pub struct LocalControls(pub Vec<Device>);

/// The grid cell the mouse last pointed at. Kept while the pointer is away,
/// so the mouse player's cursor keeps heading for the last target.
#[derive(Resource, Default)]
pub struct MouseTarget(pub Option<(u16, u16)>);

struct KeyZone {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
}

/// Default zones at opposite ends of the keyboard: WASD and the arrow keys.
const ZONES: [KeyZone; 2] = [
    KeyZone {
        up: KeyCode::KeyW,
        down: KeyCode::KeyS,
        left: KeyCode::KeyA,
        right: KeyCode::KeyD,
    },
    KeyZone {
        up: KeyCode::ArrowUp,
        down: KeyCode::ArrowDown,
        left: KeyCode::ArrowLeft,
        right: KeyCode::ArrowRight,
    },
];

/// One command per slot, read from that slot's device.
pub fn commands(
    devices: &[Device],
    keys: &ButtonInput<KeyCode>,
    mouse: Option<(u16, u16)>,
) -> Vec<Command> {
    devices
        .iter()
        .map(|device| match *device {
            Device::KeyZone(z) => ZONES.get(z).map_or(Command::Idle, |zone| {
                let axis = |neg, pos| i8::from(keys.pressed(pos)) - i8::from(keys.pressed(neg));
                let (dx, dy) = (axis(zone.left, zone.right), axis(zone.up, zone.down));
                if dx == 0 && dy == 0 {
                    Command::Idle
                } else {
                    Command::Direction { dx, dy }
                }
            }),
            Device::Mouse => mouse.map_or(Command::Idle, |(x, y)| Command::Target { x, y }),
        })
        .collect()
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseTarget>()
            .add_systems(Update, (track_mouse, grab_pointer, quit_on_escape));
    }
}

/// Esc quits the game, even mid-match with the pointer grabbed.
pub fn quit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn track_mouse(
    window: Single<&Window, With<PrimaryWindow>>,
    sim: Res<Sim>,
    mut target: ResMut<MouseTarget>,
) {
    let Some(position) = window.cursor_position() else {
        return;
    };
    let grid = (sim.0.grid.width(), sim.0.grid.height());
    target.0 = Some(GridLayout::fit(window.size(), grid).cell_at(position));
}

/// While a match with a mouse player runs and the window has focus, hide the
/// OS pointer and keep it inside the window (ADR 0004). Release it otherwise.
fn grab_pointer(
    window: Single<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    sim: Res<Sim>,
    controls: Res<LocalControls>,
) {
    let (window, mut cursor) = window.into_inner();
    let grab = window.focused && sim.0.outcome().is_none() && controls.0.contains(&Device::Mouse);
    let (visible, mode) = if grab {
        (false, CursorGrabMode::Confined)
    } else {
        (true, CursorGrabMode::None)
    };
    if cursor.visible != visible || cursor.grab_mode != mode {
        cursor.visible = visible;
        cursor.grab_mode = mode;
    }
}

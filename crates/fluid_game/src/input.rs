//! Keyboard input source (ADR 0004, 0005): each local player has a key zone on
//! the shared keyboard, turned into `Direction` commands.

use bevy::prelude::*;
use fluid_sim::Command;

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

/// One command per player slot. Slots without a key zone stay `Idle`.
pub fn keyboard_commands(keys: &ButtonInput<KeyCode>, player_count: usize) -> Vec<Command> {
    (0..player_count)
        .map(|p| match ZONES.get(p) {
            Some(zone) => {
                let axis = |neg, pos| i8::from(keys.pressed(pos)) - i8::from(keys.pressed(neg));
                let (dx, dy) = (axis(zone.left, zone.right), axis(zone.up, zone.down));
                if dx == 0 && dy == 0 {
                    Command::Idle
                } else {
                    Command::Direction { dx, dy }
                }
            }
            None => Command::Idle,
        })
        .collect()
}

//! Desktop app around `fluid_sim` (ADR 0002): local input sources feed
//! per-tick commands into a fixed-timestep simulation, and the renderer draws
//! a read-only view of the grid.

pub mod hud;
pub mod input;
pub mod layout;
pub mod render;
pub mod setup;

use bevy::prelude::*;
use fluid_sim::GameState;
use input::{LocalControls, MouseTarget};
use setup::{Screen, SetupChoices};

/// Simulation ticks per second. The real clock only decides how many ticks run
/// per frame; the simulation itself counts ticks (ADR 0002).
pub const TICK_HZ: f64 = 60.0;

/// The running match.
#[derive(Resource)]
pub struct Sim(pub GameState);

/// Start the game.
pub fn run() {
    let choices = SetupChoices::default();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fluid-game".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(TICK_HZ))
        .insert_resource(LocalControls(choices.devices.clone()))
        .insert_resource(Sim(GameState::new(choices.match_config())))
        .add_plugins(setup::SetupPlugin)
        .add_systems(FixedUpdate, tick.run_if(in_state(Screen::Match)))
        .add_plugins((render::RenderPlugin, hud::HudPlugin, input::InputPlugin))
        .run();
}

fn tick(
    mut sim: ResMut<Sim>,
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<LocalControls>,
    mouse: Res<MouseTarget>,
) {
    let commands = input::commands(&controls.0, &keys, mouse.0);
    sim.0.step(&commands);
}

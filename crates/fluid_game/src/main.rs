//! Desktop app around `fluid_sim` (ADR 0002): keyboard input sources feed
//! per-tick commands into a fixed-timestep simulation, and the renderer draws
//! a read-only view of the grid.

mod input;
mod render;

use bevy::prelude::*;
use fluid_sim::{GameState, Map, MatchConfig};

/// Simulation ticks per second. The real clock only decides how many ticks run
/// per frame; the simulation itself counts ticks (ADR 0002).
const TICK_HZ: f64 = 60.0;

/// The running match.
#[derive(Resource)]
pub struct Sim(pub GameState);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fluid-game".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(TICK_HZ))
        .insert_resource(Sim(GameState::new(MatchConfig {
            map: Map::PILLARS,
            ..MatchConfig::default()
        })))
        .add_systems(FixedUpdate, tick)
        .add_plugins(render::RenderPlugin)
        .run();
}

fn tick(mut sim: ResMut<Sim>, keys: Res<ButtonInput<KeyCode>>) {
    let commands = input::keyboard_commands(&keys, sim.0.player_count());
    sim.0.step(&commands);
}

//! Desktop app around `fluid_sim` (ADR 0002): local input sources feed
//! per-tick commands into a fixed-timestep simulation, and the renderer draws
//! a read-only view of the grid.

pub mod hud;
pub mod input;
pub mod layout;
pub mod render;

use bevy::prelude::*;
use fluid_sim::{GameState, Map, MatchConfig};
use input::{Device, LocalControls, MouseTarget};

/// Simulation ticks per second. The real clock only decides how many ticks run
/// per frame; the simulation itself counts ticks (ADR 0002).
pub const TICK_HZ: f64 = 60.0;

/// The running match.
#[derive(Resource)]
pub struct Sim(pub GameState);

/// Settings for the next match (until the setup screen exists, #8).
#[derive(Resource)]
struct Settings(MatchConfig);

/// Start the game.
pub fn run() {
    let settings = MatchConfig {
        player_count: 3,
        map: Map::PILLARS,
        match_ticks: 3 * 60 * TICK_HZ as u64,
        ..MatchConfig::default()
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fluid-game".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(TICK_HZ))
        .insert_resource(Settings(settings))
        // Until the setup screen exists (#8): WASD, arrows, mouse.
        .insert_resource(LocalControls(vec![
            Device::KeyZone(0),
            Device::KeyZone(1),
            Device::Mouse,
        ]))
        .insert_resource(Sim(GameState::new(settings)))
        .add_systems(FixedUpdate, tick)
        .add_systems(Update, rematch)
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

/// Once a match is over, Enter starts a new one with the same settings.
fn rematch(mut sim: ResMut<Sim>, settings: Res<Settings>, keys: Res<ButtonInput<KeyCode>>) {
    if sim.0.outcome().is_some() && keys.just_pressed(KeyCode::Enter) {
        sim.0 = GameState::new(settings.0);
    }
}

//! Setup screen (issue #8): grid size, players and their devices, match
//! length and map, chosen before each match. The grid behind the menu is a
//! live preview of the chosen settings.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use fluid_sim::{GameState, Map, MatchConfig};

use crate::input::{Device, LocalControls, MouseTarget};
use crate::render::PLAYER_NAMES;
use crate::{Sim, TICK_HZ};

/// Which screen the game is on.
#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Setup,
    Match,
}

/// Local input devices, in the order a new player gets them (ADR 0004):
/// two key zones and the mouse, so at most three local players.
pub const DEVICES: [Device; 3] = [Device::KeyZone(0), Device::KeyZone(1), Device::Mouse];

/// Maps to choose from.
pub const MAPS: [Map; 2] = [Map::PILLARS, Map::EMPTY];

const MIN_PLAYERS: usize = 2;
const WIDTHS: std::ops::RangeInclusive<usize> = 60..=240;
const HEIGHTS: std::ops::RangeInclusive<usize> = 40..=135;
const MINUTES: std::ops::RangeInclusive<u32> = 1..=10;

/// Everything chosen on the setup screen. Kept between matches, so after a
/// match the same settings are one Enter away.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct SetupChoices {
    pub width: usize,
    pub height: usize,
    /// One device per player slot; the length is the player count.
    pub devices: Vec<Device>,
    pub minutes: u32,
    pub map: Map,
}

impl Default for SetupChoices {
    fn default() -> Self {
        Self {
            width: 160,
            height: 90,
            devices: DEVICES[..MIN_PLAYERS].to_vec(),
            minutes: 3,
            map: Map::PILLARS,
        }
    }
}

impl SetupChoices {
    /// Why these choices can't start a match, if they can't.
    pub fn problem(&self) -> Option<&'static str> {
        let used = |d: &Device| self.devices.iter().filter(|&x| x == d).count();
        if self.devices.iter().any(|d| used(d) > 1) {
            return Some("Each device can drive only one player.");
        }
        None
    }

    /// Change the number of players, clamped to what one computer supports.
    /// New players get the first device nobody uses yet.
    pub fn set_player_count(&mut self, count: usize) {
        let count = count.clamp(MIN_PLAYERS, DEVICES.len());
        self.devices.truncate(count);
        while self.devices.len() < count {
            let free = DEVICES
                .into_iter()
                .find(|d| !self.devices.contains(d))
                .expect("fewer players than devices");
            self.devices.push(free);
        }
    }

    /// The simulation settings for a match with these choices.
    pub fn match_config(&self) -> MatchConfig {
        MatchConfig {
            width: self.width,
            height: self.height,
            player_count: self.devices.len(),
            map: self.map,
            match_ticks: u64::from(self.minutes) * 60 * TICK_HZ as u64,
            ..MatchConfig::default()
        }
    }
}

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_state::<Screen>()
            .init_resource::<SetupChoices>()
            .add_systems(
                EguiPrimaryContextPass,
                setup_menu.run_if(in_state(Screen::Setup)),
            )
            .add_systems(
                Update,
                (
                    (preview, start_on_enter).run_if(in_state(Screen::Setup)),
                    back_to_setup.run_if(in_state(Screen::Match)),
                ),
            );
    }
}

/// Show the chosen grid, map and spawns behind the menu as they change.
fn preview(choices: Res<SetupChoices>, mut sim: ResMut<Sim>) {
    let config = choices.match_config();
    if sim.0.config != config {
        sim.0 = GameState::new(config);
    }
}

fn start_on_enter(
    keys: Res<ButtonInput<KeyCode>>,
    choices: Res<SetupChoices>,
    mut start: StartMatch,
) {
    if keys.just_pressed(KeyCode::Enter) && choices.problem().is_none() {
        start.run(&choices);
    }
}

/// After a match, Enter goes back to the setup screen.
fn back_to_setup(
    keys: Res<ButtonInput<KeyCode>>,
    sim: Res<Sim>,
    mut next: ResMut<NextState<Screen>>,
) {
    if sim.0.outcome().is_some() && keys.just_pressed(KeyCode::Enter) {
        next.set(Screen::Setup);
    }
}

/// Everything that starting a match touches.
#[derive(bevy::ecs::system::SystemParam)]
struct StartMatch<'w> {
    sim: ResMut<'w, Sim>,
    controls: ResMut<'w, LocalControls>,
    mouse: ResMut<'w, MouseTarget>,
    next: ResMut<'w, NextState<Screen>>,
}

impl StartMatch<'_> {
    fn run(&mut self, choices: &SetupChoices) {
        self.sim.0 = GameState::new(choices.match_config());
        self.controls.0 = choices.devices.clone();
        self.mouse.0 = None;
        self.next.set(Screen::Match);
    }
}

fn setup_menu(
    mut contexts: EguiContexts,
    mut choices: ResMut<SetupChoices>,
    mut start: StartMatch,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("fluid-game")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            egui::Grid::new("setup")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Grid width");
                    ui.add(egui::Slider::new(&mut choices.width, WIDTHS).suffix(" cells"));
                    ui.end_row();

                    ui.label("Grid height");
                    ui.add(egui::Slider::new(&mut choices.height, HEIGHTS).suffix(" cells"));
                    ui.end_row();

                    ui.label("Match length");
                    ui.add(egui::Slider::new(&mut choices.minutes, MINUTES).suffix(" min"));
                    ui.end_row();

                    ui.label("Map");
                    egui::ComboBox::from_id_salt("map")
                        .selected_text(choices.map.name)
                        .show_ui(ui, |ui| {
                            for map in MAPS {
                                ui.selectable_value(&mut choices.map, map, map.name);
                            }
                        });
                    ui.end_row();

                    ui.label("Players");
                    let mut count = choices.devices.len();
                    ui.add(egui::Slider::new(&mut count, MIN_PLAYERS..=DEVICES.len()));
                    if count != choices.devices.len() {
                        choices.set_player_count(count);
                    }
                    ui.end_row();

                    for (p, chosen) in choices.devices.iter_mut().enumerate() {
                        ui.label(PLAYER_NAMES[p]);
                        egui::ComboBox::from_id_salt(("device", p))
                            .selected_text(device_name(*chosen))
                            .show_ui(ui, |ui| {
                                for device in DEVICES {
                                    ui.selectable_value(chosen, device, device_name(device));
                                }
                            });
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            match choices.problem() {
                Some(problem) => {
                    ui.colored_label(egui::Color32::LIGHT_RED, problem);
                }
                None => {
                    if ui.button("Start (Enter)").clicked() {
                        start.run(&choices);
                    }
                }
            }
            ui.label("Esc quits.");
        });
    Ok(())
}

fn device_name(device: Device) -> &'static str {
    match device {
        Device::KeyZone(0) => "Keys: WASD",
        Device::KeyZone(1) => "Keys: arrows",
        Device::KeyZone(_) => "Keys",
        Device::Mouse => "Mouse",
    }
}

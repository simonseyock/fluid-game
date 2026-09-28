//! Pause dialog: Esc during a match stops the simulation and asks whether to
//! continue or go back to the setup screen. Esc again continues.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::setup::Screen;

/// Whether the running match is paused.
#[derive(Resource, Default)]
pub struct Paused(pub bool);

/// Run condition: the match is not paused.
pub fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

/// Esc opens the pause dialog, or closes it again.
pub fn toggle_pause(keys: Res<ButtonInput<KeyCode>>, mut paused: ResMut<Paused>) {
    if keys.just_pressed(KeyCode::Escape) {
        paused.0 = !paused.0;
    }
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Paused>()
            .add_systems(Update, toggle_pause.run_if(in_state(Screen::Match)))
            .add_systems(
                EguiPrimaryContextPass,
                pause_menu.run_if(in_state(Screen::Match).and_then(not(not_paused))),
            )
            .add_systems(OnExit(Screen::Match), |mut paused: ResMut<Paused>| {
                paused.0 = false;
            });
    }
}

fn pause_menu(
    mut contexts: EguiContexts,
    mut paused: ResMut<Paused>,
    mut next: ResMut<NextState<Screen>>,
) -> Result {
    egui::Window::new("Paused")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Continue (Esc)").clicked() {
                    paused.0 = false;
                }
                if ui.button("Back to setup").clicked() {
                    next.set(Screen::Setup);
                }
            });
        });
    Ok(())
}

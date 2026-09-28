//! Pause dialog: Esc during a match stops the simulation and asks whether to
//! continue, go back to the setup screen, or quit. Esc again continues.

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

/// A choice made in the pause dialog. The dialog only sends it;
/// `apply_pause_choice` carries it out, so the logic is testable without a UI.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseChoice {
    Continue,
    BackToSetup,
    Quit,
}

/// Carry out the choices made in the pause dialog.
pub fn apply_pause_choice(
    mut choices: MessageReader<PauseChoice>,
    mut paused: ResMut<Paused>,
    mut next: ResMut<NextState<Screen>>,
    mut exit: MessageWriter<AppExit>,
) {
    for choice in choices.read() {
        match choice {
            PauseChoice::Continue => paused.0 = false,
            PauseChoice::BackToSetup => next.set(Screen::Setup),
            PauseChoice::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
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
            .add_message::<PauseChoice>()
            .add_systems(
                Update,
                (toggle_pause, apply_pause_choice).run_if(in_state(Screen::Match)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                pause_menu.run_if(in_state(Screen::Match).and_then(not(not_paused))),
            )
            .add_systems(OnExit(Screen::Match), |mut paused: ResMut<Paused>| {
                paused.0 = false;
            });
    }
}

fn pause_menu(mut contexts: EguiContexts, mut choices: MessageWriter<PauseChoice>) -> Result {
    egui::Window::new("Paused")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                for (label, choice) in [
                    ("Continue (Esc)", PauseChoice::Continue),
                    ("Back to setup", PauseChoice::BackToSetup),
                    ("Quit", PauseChoice::Quit),
                ] {
                    if ui.button(label).clicked() {
                        choices.write(choice);
                    }
                }
            });
        });
    Ok(())
}

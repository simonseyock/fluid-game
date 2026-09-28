//! Pause dialog: Esc during a match opens it, Esc again continues. Its
//! choices continue the match, go back to the setup screen, or quit.

use bevy::app::{App, AppExit, Update};
use bevy::input::ButtonInput;
use bevy::prelude::KeyCode;
use bevy::state::app::{AppExtStates, StatesPlugin};
use bevy::state::state::State;
use fluid_game::pause::{PauseChoice, Paused, apply_pause_choice, toggle_pause};
use fluid_game::setup::Screen;

fn tap_escape(app: &mut App) {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::Escape);
    keys.clear();
    keys.press(KeyCode::Escape);
    app.update();
}

#[test]
fn escape_pauses_and_escape_again_continues() {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<Paused>()
        .add_systems(Update, toggle_pause);
    app.update();
    assert!(
        !app.world().resource::<Paused>().0,
        "not paused without Esc"
    );
    tap_escape(&mut app);
    assert!(app.world().resource::<Paused>().0, "Esc pauses");
    tap_escape(&mut app);
    assert!(!app.world().resource::<Paused>().0, "Esc again continues");
}

/// A paused match, with the dialog's choices applied each update.
fn paused_match() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .insert_state(Screen::Match)
        .insert_resource(Paused(true))
        .add_message::<PauseChoice>()
        .add_systems(Update, apply_pause_choice);
    app.update();
    app
}

fn choose(app: &mut App, choice: PauseChoice) {
    app.world_mut().write_message(choice);
    app.update();
    // State changes apply at the start of the next update.
    app.update();
}

fn screen(app: &App) -> Screen {
    *app.world().resource::<State<Screen>>().get()
}

#[test]
fn continue_resumes_the_match() {
    let mut app = paused_match();
    choose(&mut app, PauseChoice::Continue);
    assert!(!app.world().resource::<Paused>().0);
    assert_eq!(screen(&app), Screen::Match);
    assert_eq!(app.should_exit(), None);
}

#[test]
fn back_to_setup_leaves_the_match() {
    let mut app = paused_match();
    choose(&mut app, PauseChoice::BackToSetup);
    assert_eq!(screen(&app), Screen::Setup);
    assert_eq!(app.should_exit(), None);
}

#[test]
fn quit_closes_the_game() {
    let mut app = paused_match();
    choose(&mut app, PauseChoice::Quit);
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}

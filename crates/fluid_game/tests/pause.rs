//! Esc during a match opens the pause dialog; Esc again continues.

use bevy::app::{App, Update};
use bevy::input::ButtonInput;
use bevy::prelude::KeyCode;
use fluid_game::pause::{Paused, toggle_pause};

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

//! Setup screen choices (issue #8): what they turn into, and what's allowed.

use fluid_game::TICK_HZ;
use fluid_game::input::Device;
use fluid_game::setup::SetupChoices;
use fluid_sim::{GameState, Map};

#[test]
fn choices_become_the_match_settings() {
    let choices = SetupChoices {
        width: 120,
        height: 60,
        devices: vec![Device::Mouse, Device::KeyZone(1), Device::KeyZone(0)],
        minutes: 2,
        map: Map::EMPTY,
    };
    let config = choices.match_config();
    assert_eq!((config.width, config.height), (120, 60));
    assert_eq!(config.player_count, 3);
    assert_eq!(config.match_ticks, 2 * 60 * TICK_HZ as u64);
    assert_eq!(config.map, Map::EMPTY);
    let state = GameState::new(config);
    assert_eq!((state.grid.width(), state.grid.height()), (120, 60));
    assert_eq!(state.player_count(), 3);
}

#[test]
fn defaults_are_a_valid_two_player_match() {
    let choices = SetupChoices::default();
    assert_eq!(choices.problem(), None);
    assert_eq!(choices.devices.len(), 2);
}

#[test]
fn a_device_can_drive_only_one_player() {
    let choices = SetupChoices {
        devices: vec![Device::KeyZone(0), Device::KeyZone(0)],
        ..SetupChoices::default()
    };
    assert!(choices.problem().is_some());
}

#[test]
fn adding_a_player_picks_a_free_device() {
    let mut choices = SetupChoices {
        devices: vec![Device::Mouse, Device::KeyZone(0)],
        ..SetupChoices::default()
    };
    choices.set_player_count(3);
    assert_eq!(choices.devices[2], Device::KeyZone(1));
    assert_eq!(choices.problem(), None);
    choices.set_player_count(2);
    assert_eq!(choices.devices, vec![Device::Mouse, Device::KeyZone(0)]);
}

#[test]
fn player_count_stays_within_local_limits() {
    let mut choices = SetupChoices::default();
    choices.set_player_count(1);
    assert_eq!(choices.devices.len(), 2);
    choices.set_player_count(9);
    // Two key zones and one mouse: three local players at most (ADR 0004).
    assert_eq!(choices.devices.len(), 3);
}

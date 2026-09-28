//! Text on top of the grid: the match clock while playing, and the result
//! once the match is over.

use bevy::prelude::*;
use fluid_sim::Outcome;

use crate::render::{PLAYER_COLORS, PLAYER_NAMES};
use crate::{Sim, TICK_HZ};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update_clock, update_result, place_text));
    }
}

#[derive(Component)]
struct Clock;

#[derive(Component)]
struct ResultText;

/// Where a text sits: a fraction of the window height above the centre.
#[derive(Component)]
struct Anchor(f32);

fn setup(mut commands: Commands) {
    commands.spawn((
        Clock,
        Anchor(0.45),
        Text2d::default(),
        TextFont::from_font_size(28.0),
        TextColor(Color::WHITE),
    ));
    commands.spawn((
        ResultText,
        Anchor(0.0),
        Text2d::default(),
        TextFont::from_font_size(48.0),
        TextLayout::justify(Justify::Center),
    ));
}

fn update_clock(sim: Res<Sim>, mut clock: Single<&mut Text2d, With<Clock>>) {
    let seconds = (sim.0.remaining_ticks() as f64 / TICK_HZ).ceil() as u64;
    clock.0 = format!("{}:{:02}", seconds / 60, seconds % 60);
}

fn update_result(
    sim: Res<Sim>,
    mut result: Single<(&mut Text2d, &mut TextColor, &mut Visibility), With<ResultText>>,
) {
    let (text, color, visibility) = &mut *result;
    let Some(outcome) = sim.0.outcome() else {
        **visibility = Visibility::Hidden;
        return;
    };
    **visibility = Visibility::Visible;
    let (headline, rgba) = match outcome {
        Outcome::Winner(p) => (
            format!("{} wins!", PLAYER_NAMES[p as usize]),
            PLAYER_COLORS[p as usize],
        ),
        Outcome::Tie(players) => {
            let names: Vec<&str> = players.iter().map(|&p| PLAYER_NAMES[p as usize]).collect();
            (format!("Tie: {}", names.join(" & ")), [255, 255, 255, 255])
        }
    };
    text.0 = format!("{headline}\nPress Enter for a rematch");
    color.0 = Color::srgba_u8(rgba[0], rgba[1], rgba[2], rgba[3]);
}

/// Keep texts in place, and in front of the grid, as the window resizes.
fn place_text(window: Single<&Window>, mut texts: Query<(&Anchor, &mut Transform)>) {
    for (anchor, mut transform) in &mut texts {
        transform.translation = Vec3::new(0.0, anchor.0 * window.height(), 1.0);
    }
}

//! Match clock, elimination, and the winner (ADR 0005): a player whose colour
//! hits zero cells is out, the last colour left wins, and when time runs out
//! the most cells wins (ties stay ties).

use fluid_sim::{Cell, Command, GameState, MatchConfig, Outcome, PlayerId};

fn config(players: usize, match_ticks: u64) -> MatchConfig {
    MatchConfig {
        width: 60,
        height: 60,
        player_count: players,
        match_ticks,
        ..MatchConfig::default()
    }
}

/// Remove every cell of `player`'s fluid, or only the first `limit` of them.
fn remove_fluid(s: &mut GameState, player: PlayerId, limit: usize) {
    let (w, h) = (s.grid.width(), s.grid.height());
    let mut removed = 0;
    for y in 0..h {
        for x in 0..w {
            if removed < limit && s.grid.get(x, y) == Cell::Fluid(player) {
                s.grid.set(x, y, Cell::Empty);
                removed += 1;
            }
        }
    }
}

fn idle(s: &mut GameState, ticks: u64) {
    let commands = vec![Command::Idle; s.player_count()];
    for _ in 0..ticks {
        s.step(&commands);
    }
}

#[test]
fn match_runs_until_decided() {
    let mut s = GameState::new(config(2, 100));
    assert_eq!(s.remaining_ticks(), 100);
    idle(&mut s, 40);
    assert_eq!(s.remaining_ticks(), 60);
    assert_eq!(s.outcome(), None);
}

#[test]
fn eliminated_players_are_detected() {
    let mut s = GameState::new(config(3, 1000));
    remove_fluid(&mut s, 2, usize::MAX);
    assert!(s.is_eliminated(2));
    assert!(!s.is_eliminated(0) && !s.is_eliminated(1));
    // Two colours left: the match goes on.
    assert_eq!(s.outcome(), None);
}

#[test]
fn last_colour_left_wins() {
    let mut s = GameState::new(config(3, 1000));
    remove_fluid(&mut s, 2, usize::MAX);
    remove_fluid(&mut s, 0, usize::MAX);
    assert_eq!(s.outcome(), Some(Outcome::Winner(1)));
}

#[test]
fn at_timeout_the_most_cells_wins() {
    let mut s = GameState::new(config(2, 50));
    remove_fluid(&mut s, 0, 5);
    idle(&mut s, 49);
    assert_eq!(s.outcome(), None, "not over before the clock runs out");
    idle(&mut s, 1);
    assert_eq!(s.remaining_ticks(), 0);
    assert_eq!(s.outcome(), Some(Outcome::Winner(1)));
}

#[test]
fn at_timeout_equal_cells_is_a_tie() {
    let mut s = GameState::new(config(3, 50));
    // Players 0 and 1 keep equal fluid; player 2 falls behind.
    remove_fluid(&mut s, 2, 5);
    idle(&mut s, 50);
    assert_eq!(s.outcome(), Some(Outcome::Tie(vec![0, 1])));
}

#[test]
fn nothing_changes_after_the_match_is_over() {
    let mut s = GameState::new(config(2, 10));
    idle(&mut s, 10);
    let over = s.clone();
    s.step(&[Command::Direction { dx: 1, dy: 0 }, Command::Idle]);
    assert_eq!(s, over);
}

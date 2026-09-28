//! Helpers shared by the integration tests.

use fluid_sim::MatchConfig;

/// A small grid so tests run fast.
pub fn small() -> MatchConfig {
    MatchConfig {
        width: 40,
        height: 30,
        ..MatchConfig::default()
    }
}

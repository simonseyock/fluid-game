# fluid-game

A native, pixel-rendered multiplayer game for two or more players: two on one
keyboard plus one on the mouse (network play planned). Each player steers a cursor that their neon fluid
follows. Encircle enemy fluid to turn it your color. Be the last color standing,
or be ahead when time runs out.

Built with **Rust + Bevy** (see
[`architecture/0001-native-language.md`](architecture/0001-native-language.md)).
See [`MISSION.md`](MISSION.md) for the full vision and
[`architecture/`](architecture/) for design decisions.

## Development setup (Ubuntu/Debian)

1. Install the Rust toolchain via rustup:
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. That's it for now. The game builds with Bevy's X11 backend only (no audio,
   no gamepads), so no extra system packages are needed. On a Wayland desktop
   it runs through XWayland.

The first build compiles Bevy and takes several minutes. Later builds are fast.

## Layout

- `crates/fluid_sim`: the deterministic simulation core (ADR 0002). Plain Rust,
  no Bevy. Tests are in `crates/fluid_sim/tests/`.
- `crates/fluid_game`: the Bevy desktop app: input, rendering, fixed-timestep loop.

## Run

```sh
cargo run -p fluid_game
```

Controls: player 1 (cyan) uses **WASD**, player 2 (magenta) uses the **arrow keys**.

## Test

```sh
cargo test -p fluid_sim          # simulation rules
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

## Contributing

See [`AGENTS.md`](AGENTS.md) for the workflow (tracer-bullet slices) and task
tracking (Gitea issues).

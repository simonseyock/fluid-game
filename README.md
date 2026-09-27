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
2. Install Bevy's Linux build dependencies:
   ```sh
   sudo apt install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
   ```

Run and test commands will be added here once the first code slice exists.

## Contributing

See [`AGENTS.md`](AGENTS.md) for the workflow (tracer-bullet slices) and task
tracking (Gitea issues).

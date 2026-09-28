# Mission

**fluid-game** is a multiplayer game for two or more players. On one computer,
two players share the keyboard and a third can play with the mouse. Later others
can join over the network. Each player steers a
cursor, and their colored fluid flows after it. A player takes territory by
encircling enemy fluid: fluid that is fully enclosed switches to the encircling
player's color. A player whose color is wiped out is eliminated, and the last
color standing wins. If no one wins that way before time runs out, the match is
decided on the clock.

## Who it's for

Friends at the same computer (two on the keyboard, optionally a third on the
mouse) who want a quick, competitive, visual match. Later also players on other
devices, or a mix: players on one computer against others over the network.

## Core principles

- **Local first, network-ready.** The game ships as local multiplayer on one
  computer, but it must be built so networked and mixed local/network play can
  be added later without a rewrite. Keep the game state and rules separate from
  where input comes from.
- **Any number of players, within limits.** The player count is chosen at setup. It's capped by how many distinct neon
  colors work in practice, and locally by the input devices (two keyboard zones
  plus one mouse).
- **Fair across input devices.** Keyboard, mouse, and (later) touch players
  must have an equal chance. No device may move its cursor faster or more
  freely than the game rules allow.
- **Pixel-rendered.** The world is a grid of pixels. Players set the grid size
  in the level setup screen before a match.
- **Obstacles shape play.** Maps contain obstacles that the fluid has to flow
  around.
- **Neon on dark.** Vibrant neon colors on black or dark backgrounds. Every
  visual choice should serve that look and keep every player's color easy
  to tell apart.
- **Native.** The game runs as a native desktop application.
- **Android later, network only.** Android is a future target. There, the game
  is played only as network multiplayer, one player per device, with no
  shared-device mode. Desktop and Android should be able to play the same
  networked match.

## Non-goals (for now)

- Networked play. It's a planned extension, not part of the first version,
  but no design may rule it out.
- Android. It's a planned target that depends on networked play. No design may
  rule it out.
- Local (shared-device) multiplayer on Android.

# 0004. Platform-specific input sources and Android as a network-only target

**Status:** Accepted

## Context

MISSION.md names Android as a future target. On Android, only network
multiplayer is possible: there's no keyboard to share, so each device controls
exactly one player. ADR 0002 already separates the simulation core from input
sources. ADR 0005 defines a match as N player slots, each filled by a player on
the local device or on a remote peer. ADR 0006 defines the player command that
every input source produces. This ADR decides how platforms supply input to
those slots and which setups each platform allows.

## Decision

- **Input sources are per platform, and each one fills one or more player
  slots on this device.** An input source turns raw device input into
  per-tick commands for the slots it owns:
  - Desktop keyboard: fills **up to two** local slots, each with its own key
    zone on the shared keyboard (ADR 0005). Produces *direction* commands.
  - Desktop mouse: fills **at most one** local slot. The operating system
    merges all connected mice into a single pointer, so a second mouse can't
    be a separate player. Produces *target* commands (ADR 0006).
  - Android touch: fills **exactly one** local slot. Touch steers the cursor
    toward the finger, producing *target* commands like the mouse.
  - Network: fills every slot owned by a remote peer with commands received
    from that peer.
- **Allowed setups depend on the platform:**
  - Desktop: any mix of local and remote players, with at most three local
    (two keyboard + one mouse), from all local (no network) to one local
    player with all others remote.
  - Android: exactly one local player per device. Every other slot is remote,
    so an Android match always needs the network. The setup screen offers only
    that option.
- **The simulation core and network protocol are identical on all platforms,**
  so Android devices and desktops (each with any number of local players) can
  play the same networked match.
- Bevy-specific input code (keyboard, mouse, touch) lives only in the
  input-source layer, never in the simulation crate. That includes converting
  window/screen coordinates to grid cells.

## Rationale

With input sources behind one interface, adding touch or a network peer is a new
source, not a change to game rules. The simulation doesn't know or care how many
players share a device. Sharing one protocol and one deterministic core means
cross-platform play comes for free once networking exists. Limiting Android to
one player per device avoids designing a cramped multi-player touch layout on a
single phone.

## Consequences

- Android support can't ship before network multiplayer exists.
- Determinism (ADR 0002) must hold across CPU architectures (x86_64 desktop vs
  ARM64 Android). That's a strong argument for integer/fixed-point math in the
  simulation core.
- Mouse and touch share the target-command path, so touch mostly reuses the
  mouse work. UI scaling for phones still needs its own slice.
- During a match the desktop window should grab and hide the OS mouse pointer,
  so the mouse player doesn't click outside the window or lose focus.
- Android builds need the Android SDK/NDK and Rust Android targets. Setup
  gets documented in the README when that work starts.

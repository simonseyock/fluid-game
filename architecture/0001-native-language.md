# 0001. Language and game engine / libraries

**Status:** Accepted

## Context

MISSION.md and ADRs 0002/0003 set these requirements for the stack:

- **Native desktop** application.
- **Pixel grid rendering:** a whole cell grid rewritten every frame (per-pixel
  texture upload or GPU shader), with a configurable pixel size.
- **Two players on one keyboard:** raw key-state polling for many keys held at once.
- **Neon on dark:** glow/bloom post-processing is what sells the look.
- **Level setup screen:** some UI (sliders, buttons, map picker).
- **Network-ready:** a deterministic simulation (ADR 0002). That favors a
  rollback or lockstep netcode library and integer/fixed-point math.
- **Simulation speed:** small pixel sizes mean hundreds of thousands to millions
  of cells per tick, so the rules need native-speed CPU code (a GPU-only
  simulation would give up cross-machine determinism).

## Options, by ecosystem

### Rust
- **Engines/libraries:** **Bevy** (ECS engine with built-in HDR + bloom, active
  community), **macroquad** (small and simple, good for pixel games), **pixels**
  + winit (raw framebuffer), ggez, Fyrox.
- **UI:** `egui` (via `bevy_egui` / `egui-macroquad`), or Bevy UI.
- **Networking:** **GGRS** (a Rust version of GGPO rollback) with `bevy_ggrs`,
  `matchbox` (P2P/WebRTC), `lightyear`, `renet`.
- **Determinism:** `fixed` crate for fixed-point math. Bevy + GGRS setups
  already expect a deterministic simulation.
- **Cons:** Bevy's API still changes between releases. Compile times are long.

### C++
- **Engines/libraries:** **raylib** (simple, shaders, pixel-friendly), **SDL3**,
  SFML, bgfx. Godot (via GDExtension) and Unreal are options but heavy for this game.
- **UI:** Dear ImGui, raygui.
- **Networking:** **GGPO** (the original rollback library), ENet, Valve
  GameNetworkingSockets, yojimbo.
- **Determinism:** full control, but guaranteed only if you enforce it yourself.
- **Cons:** build systems and dependency management take the most work. Memory
  safety is up to you.

### C# 
- **Engines/libraries:** **Godot (C#)**, **MonoGame** / FNA, Stride, Unity.
- **UI:** Godot Control nodes (excellent), Myra/ImGui.NET for MonoGame.
- **Networking:** LiteNetLib, Godot's high-level multiplayer API, Backdash
  (rollback). Unity has Photon Quantum (deterministic, commercial).
- **Cons:** Godot's C# build doesn't export to web. Unity's licensing history
  is a risk.

### GDScript (Godot)
- **Engine:** **Godot 4**: built-in glow (WorldEnvironment), `Image`/
  `ImageTexture` for pixel grids, excellent UI tools, input maps.
- **Networking:** built-in high-level multiplayer API, and a community rollback
  add-on (godot-rollback-netcode).
- **Cons:** GDScript is too slow for a large per-cell simulation. The core would
  need moving to C# or C++ (GDExtension) anyway, which means two languages.

### Go
- **Engine:** **Ebitengine**: a mature 2D engine, a strong fit for pixel
  games (`WritePixels`, Kage shaders).
- **UI/Networking:** ebitenui, standard library networking. No mainstream
  rollback library.
- **Cons:** a smaller game ecosystem. Garbage-collection pauses are usually fine
  at this scale.

### Java / Kotlin
- **Engines/libraries:** **libGDX**, LWJGL, KorGE (Kotlin).
- **UI/Networking:** Scene2D; KryoNet/Netty. No mainstream rollback library.
- **Cons:** the game ecosystem is shrinking. Desktop packaging needs a bundled JRE.

### Lua (LÖVE)
- **Engine:** **LÖVE 2D**: pixel-friendly, GLSL shaders, ENet built in.
- **Cons:** a per-cell simulation in LuaJIT runs into limits at small pixel
  sizes. Dynamic typing makes it harder to keep a large core deterministic.

### Zig / Odin
- raylib and SDL bindings (Odin ships them in `vendor:`). Mach engine (Zig) is
  still young. Small ecosystems, few ready-made netcode or UI options.

### Python
- pygame-ce, arcade, pyglet. Too slow for per-pixel simulation without numpy or
  numba contortions. Fine for prototypes only.

## Decision

**Rust + Bevy.** GGRS (`bevy_ggrs`) is the intended path for networking
later, and egui (`bevy_egui`) for the setup screen.

## Rationale

Rust + Bevy is the only ecosystem that ready-made covers **every** requirement:
- native-speed simulation code
- built-in bloom for the neon look
- a GPU texture updated from a CPU grid for the pixel rendering
- egui for the setup screen
- above all, a maintained rollback-netcode integration (`bevy_ggrs`) designed
  around the same deterministic, input-driven core that ADR 0002 already requires

Good alternatives:
- **Rust + macroquad**, if Bevy's churn and ECS feel too heavy (GGRS still works,
  but you build more by hand).
- **C++ + raylib (+ GGPO)**, for the simplest API and the proven original
  rollback library, at the cost of build and safety effort.
- **Godot 4 with the simulation in C#**, for the best editor, UI, and glow
  tooling, at the cost of a split codebase and hand-built determinism.

## Consequences

- The simulation core should be a plain library (crate/module) with no engine
  imports, so it can be unit-tested headless and reused by a future server or
  rollback session.
- The simulation core lives in its own crate with no Bevy dependency. The Bevy
  app is a separate crate that depends on it.
- Linux builds need the system packages Bevy requires (see README).
- Bevy's API changes between releases, so pin the Bevy version in `Cargo.toml`
  and plan upgrades as deliberate slices.

/// One player's input for one tick (ADR 0006). Fixed-size plain data, so it can
/// be sent over the network as-is later.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Command {
    #[default]
    Idle,
    /// Held direction from a keyboard zone. Each component is -1, 0 or 1.
    Direction { dx: i8, dy: i8 },
    /// Absolute grid cell from a mouse or touch. The cursor moves toward it at
    /// the capped cursor speed, never jumping.
    Target { x: u16, y: u16 },
}

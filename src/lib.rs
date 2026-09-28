//! Rust Idle Game - A base template for idle games in Rust
//!
//! This crate provides a clean, extensible architecture for building idle games
//! using an ECS (Entity Component System) pattern with Bevy ECS.

pub mod components;
pub mod game;
pub mod resources;
pub mod save;
pub mod systems;
pub mod ui;

use bevy_ecs::prelude::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::components::*;
    pub use crate::game::GameState;
    pub use crate::resources::*;
    pub use crate::save::*;
    pub use crate::systems::*;
    pub use crate::ui::*;
    pub use bevy_ecs::prelude::*;
    pub use serde::{Deserialize, Serialize};
}
//! Components module - ECS components for the idle game

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource types in the game
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component, Reflect
)]
#[reflect(Component)]
pub enum ResourceType {
    Gold,
    Wood,
    Stone,
    Food,
    Science,
    Magic,
}

impl ResourceType {
    pub fn all() -> [ResourceType; 6] {
        [
            ResourceType::Gold,
            ResourceType::Wood,
            ResourceType::Stone,
            ResourceType::Food,
            ResourceType::Science,
            ResourceType::Magic,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Gold => "Gold",
            ResourceType::Wood => "Wood",
            ResourceType::Stone => "Stone",
            ResourceType::Food => "Food",
            ResourceType::Science => "Science",
            ResourceType::Magic => "Magic",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ResourceType::Gold => "💰",
            ResourceType::Wood => "🪵",
            ResourceType::Stone => "🪨",
            ResourceType::Food => "🍎",
            ResourceType::Science => "🔬",
            ResourceType::Magic => "✨",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            ResourceType::Gold => ratatui::style::Color::Yellow,
            ResourceType::Wood => ratatui::style::Color::Green,
            ResourceType::Stone => ratatui::style::Color::Gray,
            ResourceType::Food => ratatui::style::Color::Red,
            ResourceType::Science => ratatui::style::Color::Blue,
            ResourceType::Magic => ratatui::style::Color::Magenta,
        }
    }
}

/// Unique identifier for upgrades
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct UpgradeId(pub String);

impl UpgradeId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// Unique identifier for buildings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct BuildingId(pub String);

impl BuildingId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// State of an upgrade
#[derive(Serialize, Deserialize, Debug, Clone, Default, Component)]
pub struct UpgradeState {
    pub level: u32,
    pub total_levels_purchased: u32,
}

/// State of a building
#[derive(Serialize, Deserialize, Debug, Clone, Default, Component)]
pub struct BuildingState {
    pub level: u32,
    pub total_built: u32,
}

/// Effect of an upgrade
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UpgradeEffect {
    AddResourceRate(ResourceType, f64),
    MultiplyResourceRate(ResourceType, f64),
    UnlockResource(ResourceType),
    AddBuildingProduction(BuildingId, ResourceType, f64),
    GlobalMultiplier(f64),
}

/// Upgrade definition
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Upgrade {
    pub id: UpgradeId,
    pub name: String,
    pub description: String,
    pub cost: Vec<(ResourceType, f64)>,
    pub cost_growth: f64,
    pub effect: UpgradeEffect,
    pub max_level: u32,
    pub prerequisites: Vec<UpgradeId>,
    pub hidden: bool,
}

impl Default for Upgrade {
    fn default() -> Self {
        Self {
            id: UpgradeId(String::new()),
            name: String::new(),
            description: String::new(),
            cost: Vec::new(),
            cost_growth: 1.15,
            effect: UpgradeEffect::AddResourceRate(ResourceType::Gold, 0.0),
            max_level: 1,
            prerequisites: Vec::new(),
            hidden: false,
        }
    }
}

/// Building definition
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Building {
    pub id: BuildingId,
    pub name: String,
    pub description: String,
    pub base_cost: Vec<(ResourceType, f64)>,
    pub cost_growth: f64,
    pub production: Vec<(ResourceType, f64)>,
    pub max_level: u32,
    pub prerequisites: Vec<UpgradeId>,
    pub hidden: bool,
}

impl Default for Building {
    fn default() -> Self {
        Self {
            id: BuildingId(String::new()),
            name: String::new(),
            description: String::new(),
            base_cost: Vec::new(),
            cost_growth: 1.15,
            production: Vec::new(),
            max_level: 1,
            prerequisites: Vec::new(),
            hidden: false,
        }
    }
}

/// Registry of all upgrades
#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub struct UpgradeRegistry {
    pub upgrades: HashMap<UpgradeId, Upgrade>,
}

impl UpgradeRegistry {
    pub fn register(&mut self, upgrade: Upgrade) {
        self.upgrades.insert(upgrade.id.clone(), upgrade);
    }

    pub fn get(&self, id: &UpgradeId) -> Option<&Upgrade> {
        self.upgrades.get(id)
    }

    pub fn all(&self) -> Vec<&Upgrade> {
        self.upgrades.values().collect()
    }

    pub fn available(&self, state: &crate::game::GameState) -> Vec<&Upgrade> {
        self.upgrades
            .values()
            .filter(|u| {
                !u.hidden
                    && u.prerequisites.iter().all(|p| {
                        state.upgrades.get(p).map(|s| s.level > 0).unwrap_or(false)
                    })
            })
            .collect()
    }
}

/// Registry of all buildings
#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub struct BuildingRegistry {
    pub buildings: HashMap<BuildingId, Building>,
}

impl BuildingRegistry {
    pub fn register(&mut self, building: Building) {
        self.buildings.insert(building.id.clone(), building);
    }

    pub fn get(&self, id: &BuildingId) -> Option<&Building> {
        self.buildings.get(id)
    }

    pub fn all(&self) -> Vec<&Building> {
        self.buildings.values().collect()
    }

    pub fn available(&self, state: &crate::game::GameState) -> Vec<&Building> {
        self.buildings
            .values()
            .filter(|b| {
                !b.hidden
                    && b.prerequisites.iter().all(|p| {
                        state.upgrades.get(p).map(|s| s.level > 0).unwrap_or(false)
                    })
            })
            .collect()
    }
}

/// UI state for the terminal interface
#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub struct UiState {
    pub current_panel: Panel,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub confirmation_dialog: Option<ConfirmationDialog>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Resources,
    Upgrades,
    Buildings,
    Prestige,
    Statistics,
    Settings,
}

impl Default for Panel {
    fn default() -> Self {
        Panel::Resources
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: ConfirmationAction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfirmationAction {
    Prestige,
    ResetGame,
    DeleteSave,
}

/// Click component for clicker mechanic
#[derive(Component, Default)]
pub struct Clickable {
    pub resource: ResourceType,
    pub amount: f64,
    pub cooldown: f32,
}

/// Production component for buildings
#[derive(Component)]
pub struct Producer {
    pub outputs: Vec<(ResourceType, f64)>, // (resource, amount per second)
}

/// Timer component for periodic events
#[derive(Component)]
pub struct Timer {
    pub duration: f32,
    pub elapsed: f32,
    pub repeating: bool,
}

impl Timer {
    pub fn new(duration: f32, repeating: bool) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            }
            true
        } else {
            false
        }
    }
}

/// Offline progress component
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct OfflineProgress {
    pub last_save_time: chrono::DateTime<chrono::Utc>,
    pub max_offline_hours: u64,
}
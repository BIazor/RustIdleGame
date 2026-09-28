//! Resources module - ECS Resources for the game

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Formatting utilities for numbers
pub fn format_number(value: f64, scientific: bool) -> String {
    if value.is_nan() || value.is_infinite() {
        return "∞".to_string();
    }

    if scientific && value >= 1_000_000.0 {
        format!("{:.2e}", value)
    } else if value >= 1_000_000_000_000.0 {
        format!("{:.2}T", value / 1_000_000_000_000.0)
    } else if value >= 1_000_000_000.0 {
        format!("{:.2}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2}K", value / 1_000.0)
    } else if value.fract() == 0.0 && value < 1_000_000.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}

pub fn format_rate(rate: f64, scientific: bool) -> String {
    format!("/s ({})", format_number(rate, scientific))
}

/// Time resource for tracking game time
#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub struct GameTime {
    pub total_time: f64,
    pub session_time: f64,
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
}

impl GameTime {
    pub fn update(&mut self, delta: f32) {
        self.total_time += delta as f64;
        self.session_time += delta as f64;
        self.last_update = Some(chrono::Utc::now());
    }

    pub fn format_time(&self) -> String {
        let secs = self.total_time as u64;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, mins)
        } else if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m {}s", mins, secs)
        }
    }
}

/// Resource for tracking clicker state
#[derive(Resource, Default, Debug, Clone)]
pub struct ClickState {
    pub last_click: Option<std::time::Instant>,
    pub clicks_per_second: f64,
    pub total_clicks: u64,
}

impl ClickState {
    pub fn register_click(&mut self) {
        let now = std::time::Instant::now();
        if let Some(last) = self.last_click {
            let elapsed = now.duration_since(last).as_secs_f64();
            if elapsed > 0.0 {
                self.clicks_per_second = 1.0 / elapsed;
            }
        }
        self.last_click = Some(now);
        self.total_clicks += 1;
    }
}

/// Configuration resource loaded from file
#[derive(Resource, Serialize, Deserialize, Debug, Clone)]
pub struct GameConfig {
    pub starting_resources: HashMap<String, f64>,
    pub base_tick_rate: u64,
    pub prestige_formula: PrestigeFormula,
    pub building_cost_formula: CostFormula,
    pub upgrade_cost_formula: CostFormula,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PrestigeFormula {
    SquareRoot,
    Logarithmic,
    Linear,
    Custom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum CostFormula {
    Exponential { base: f64 },
    Polynomial { degree: f64 },
    Linear,
}

impl Default for GameConfig {
    fn default() -> Self {
        let mut starting_resources = HashMap::new();
        starting_resources.insert("Gold".to_string(), 0.0);
        starting_resources.insert("Wood".to_string(), 0.0);

        Self {
            starting_resources,
            base_tick_rate: 100,
            prestige_formula: PrestigeFormula::SquareRoot,
            building_cost_formula: CostFormula::Exponential { base: 1.15 },
            upgrade_cost_formula: CostFormula::Exponential { base: 1.15 },
        }
    }
}

/// Achievement system resource
#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub struct Achievements {
    pub unlocked: HashMap<String, AchievementState>,
    pub definitions: Vec<AchievementDef>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AchievementDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AchievementCondition,
    pub reward: AchievementReward,
    pub hidden: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AchievementCondition {
    ResourceTotal(ResourceType, f64),
    ResourceRate(ResourceType, f64),
    BuildingLevel(BuildingId, u32),
    UpgradeLevel(UpgradeId, u32),
    PrestigeLevel(u32),
    PlayTime(f64), // seconds
    TotalClicks(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AchievementReward {
    Resource(ResourceType, f64),
    GlobalMultiplier(f64),
    UnlockUpgrade(UpgradeId),
    UnlockBuilding(BuildingId),
    Cosmetic(String), // title, badge, etc.
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AchievementState {
    pub unlocked: bool,
    pub unlocked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub progress: f64, // 0.0 to 1.0
}

impl Achievements {
    pub fn check_conditions(&mut self, state: &crate::game::GameState) {
        for def in &self.definitions.clone() {
            let achieved = match &def.condition {
                AchievementCondition::ResourceTotal(res, target) => {
                    state.statistics.total_gained.get(res).copied().unwrap_or(0.0) >= *target
                }
                AchievementCondition::ResourceRate(res, target) => {
                    state.get_rate(*res) >= *target
                }
                AchievementCondition::BuildingLevel(bld, target) => {
                    state.buildings.get(bld).map(|s| s.level).unwrap_or(0) >= *target
                }
                AchievementCondition::UpgradeLevel(upg, target) => {
                    state.upgrades.get(upg).map(|s| s.level).unwrap_or(0) >= *target
                }
                AchievementCondition::PrestigeLevel(target) => {
                    state.prestige.level >= *target
                }
                AchievementCondition::PlayTime(target) => {
                    state.statistics.total_playtime >= *target
                }
                AchievementCondition::TotalClicks(target) => {
                    // Would need click state
                    false
                }
            };

            let entry = self.unlocked.entry(def.id.clone()).or_default();
            if achieved && !entry.unlocked {
                entry.unlocked = true;
                entry.unlocked_at = Some(chrono::Utc::now());
                // Apply reward
                self.apply_reward(&def.reward, state);
            }
            entry.progress = self.calculate_progress(&def.condition, state);
        }
    }

    fn apply_reward(&mut self, reward: &AchievementReward, state: &mut crate::game::GameState) {
        match reward {
            AchievementReward::Resource(res, amount) => {
                state.add_resource(*res, *amount);
            }
            AchievementReward::GlobalMultiplier(mult) => {
                for res in [
                    ResourceType::Gold,
                    ResourceType::Wood,
                    ResourceType::Stone,
                    ResourceType::Food,
                    ResourceType::Science,
                    ResourceType::Magic,
                ] {
                    let current = state.get_rate(res);
                    state.set_rate(res, current * mult);
                }
            }
            AchievementReward::UnlockUpgrade(id) => {
                // Upgrade becomes visible/available
            }
            AchievementReward::UnlockBuilding(id) => {
                // Building becomes visible/available
            }
            AchievementReward::Cosmetic(_) => {
                // Visual only
            }
        }
    }

    fn calculate_progress(&self, condition: &AchievementCondition, state: &crate::game::GameState) -> f64 {
        match condition {
            AchievementCondition::ResourceTotal(res, target) => {
                let current = state.statistics.total_gained.get(res).copied().unwrap_or(0.0);
                (current / target).min(1.0)
            }
            AchievementCondition::ResourceRate(res, target) => {
                let current = state.get_rate(*res);
                (current / target).min(1.0)
            }
            AchievementCondition::BuildingLevel(bld, target) => {
                let current = state.buildings.get(bld).map(|s| s.level).unwrap_or(0) as f64;
                (current / *target as f64).min(1.0)
            }
            AchievementCondition::UpgradeLevel(upg, target) => {
                let current = state.upgrades.get(upg).map(|s| s.level).unwrap_or(0) as f64;
                (current / *target as f64).min(1.0)
            }
            AchievementCondition::PrestigeLevel(target) => {
                (state.prestige.level as f64 / *target as f64).min(1.0)
            }
            AchievementCondition::PlayTime(target) => {
                (state.statistics.total_playtime / target).min(1.0)
            }
            AchievementCondition::TotalClicks(target) => {
                0.0 // Would need click state
            }
        }
    }
}
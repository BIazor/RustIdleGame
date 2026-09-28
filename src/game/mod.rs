//! Game module - Core game state and logic

pub mod prelude {
    pub use crate::components::*;
    pub use crate::resources::*;
    pub use crate::systems::*;
    pub use bevy_ecs::prelude::*;
    pub use nalgebra::Vector2;
    pub use serde::{Deserialize, Serialize};
    pub use std::time::Duration;
}

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::components::*;
use crate::resources::*;
use crate::systems::*;

/// Main game state container
#[derive(Resource, Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub resources: HashMap<ResourceType, f64>,
    pub resource_rates: HashMap<ResourceType, f64>,
    pub upgrades: HashMap<UpgradeId, UpgradeState>,
    pub buildings: HashMap<BuildingId, BuildingState>,
    pub prestige: PrestigeState,
    pub statistics: Statistics,
    pub settings: GameSettings,
    #[serde(skip)]
    pub tick_accumulator: f32,
}

impl Default for GameState {
    fn default() -> Self {
        let mut resources = HashMap::new();
        resources.insert(ResourceType::Gold, 0.0);
        resources.insert(ResourceType::Wood, 0.0);
        resources.insert(ResourceType::Stone, 0.0);
        resources.insert(ResourceType::Food, 0.0);
        resources.insert(ResourceType::Science, 0.0);
        resources.insert(ResourceType::Magic, 0.0);

        let mut resource_rates = HashMap::new();
        resource_rates.insert(ResourceType::Gold, 0.1);
        resource_rates.insert(ResourceType::Wood, 0.05);
        resource_rates.insert(ResourceType::Stone, 0.02);
        resource_rates.insert(ResourceType::Food, 0.0);
        resource_rates.insert(ResourceType::Science, 0.0);
        resource_rates.insert(ResourceType::Magic, 0.0);

        Self {
            resources,
            resource_rates,
            upgrades: HashMap::new(),
            buildings: HashMap::new(),
            prestige: PrestigeState::default(),
            statistics: Statistics::default(),
            settings: GameSettings::default(),
            tick_accumulator: 0.0,
        }
    }
}

impl GameState {
    pub fn get_resource(&self, resource: ResourceType) -> f64 {
        *self.resources.get(&resource).unwrap_or(&0.0)
    }

    pub fn add_resource(&mut self, resource: ResourceType, amount: f64) {
        *self.resources.entry(resource).or_insert(0.0) += amount;
        self.statistics.total_gained.entry(resource).or_insert(0.0) += amount.max(0.0);
    }

    pub fn spend_resource(&mut self, resource: ResourceType, amount: f64) -> bool {
        let current = self.get_resource(resource);
        if current >= amount {
            self.add_resource(resource, -amount);
            true
        } else {
            false
        }
    }

    pub fn get_rate(&self, resource: ResourceType) -> f64 {
        *self.resource_rates.get(&resource).unwrap_or(&0.0)
    }

    pub fn set_rate(&mut self, resource: ResourceType, rate: f64) {
        self.resource_rates.insert(resource, rate);
    }

    pub fn modify_rate(&mut self, resource: ResourceType, delta: f64) {
        let current = self.get_rate(resource);
        self.set_rate(resource, current + delta);
    }
}

/// Prestige system state
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PrestigeState {
    pub level: u32,
    pub points: f64,
    pub total_points_earned: f64,
    pub multipliers: HashMap<ResourceType, f64>,
}

impl PrestigeState {
    pub fn get_multiplier(&self, resource: ResourceType) -> f64 {
        *self.multipliers.get(&resource).unwrap_or(&1.0)
    }
}

/// Game statistics tracking
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Statistics {
    pub total_playtime: f64,
    pub total_gained: HashMap<ResourceType, f64>,
    pub total_spent: HashMap<ResourceType, f64>,
    pub upgrades_purchased: u32,
    pub buildings_built: u32,
    pub prestiges: u32,
    pub ticks_processed: u64,
}

/// Game settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSettings {
    pub auto_save_interval: u64, // seconds
    pub show_numbers_scientific: bool,
    pub tick_rate: u64, // ms
    pub offline_progress: bool,
    pub max_offline_hours: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            auto_save_interval: 30,
            show_numbers_scientific: true,
            tick_rate: 100,
            offline_progress: true,
            max_offline_hours: 24,
        }
    }
}

/// Main Game struct that manages the ECS world and game loop
pub struct Game {
    world: World,
    schedule: Schedule,
    last_save: std::time::Instant,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // Insert resources
        world.insert_resource(GameState::default());
        world.insert_resource(UpgradeRegistry::default());
        world.insert_resource(BuildingRegistry::default());
        world.insert_resource(UiState::default());

        // Initialize registries
        Self::register_upgrades(&mut world);
        Self::register_buildings(&mut world);

        // Build schedule
        schedule.add_systems((
            resource_production_system,
            building_production_system,
            upgrade_effect_system,
            prestige_calculation_system,
            statistics_system,
            auto_save_system,
        ).chain());

        Self {
            world,
            schedule,
            last_save: std::time::Instant::now(),
        }
    }

    fn register_upgrades(world: &mut World) {
        let mut registry = world.get_resource_mut::<UpgradeRegistry>().unwrap();

        // Gold upgrades
        registry.register(Upgrade {
            id: UpgradeId("gold_clicker"),
            name: "Golden Touch".to_string(),
            description: "Clicking generates 1 gold".to_string(),
            cost: vec![(ResourceType::Gold, 10.0)],
            effect: UpgradeEffect::AddResourceRate(ResourceType::Gold, 1.0),
            max_level: 10,
            ..Default::default()
        });

        registry.register(Upgrade {
            id: UpgradeId("gold_multiplier"),
            name: "Alchemy".to_string(),
            description: "Gold production +50%".to_string(),
            cost: vec![(ResourceType::Gold, 100.0)],
            effect: UpgradeEffect::MultiplyResourceRate(ResourceType::Gold, 1.5),
            max_level: 5,
            ..Default::default()
        });

        // Wood upgrades
        registry.register(Upgrade {
            id: UpgradeId("wood_clicker"),
            name: "Woodcutter".to_string(),
            description: "Clicking generates 1 wood".to_string(),
            cost: vec![(ResourceType::Wood, 10.0), (ResourceType::Gold, 50.0)],
            effect: UpgradeEffect::AddResourceRate(ResourceType::Wood, 1.0),
            max_level: 10,
            ..Default::default()
        });

        // Science upgrades
        registry.register(Upgrade {
            id: UpgradeId("science_unlock"),
            name: "Scholar".to_string(),
            description: "Unlocks science production".to_string(),
            cost: vec![(ResourceType::Gold, 500.0), (ResourceType::Wood, 200.0)],
            effect: UpgradeEffect::UnlockResource(ResourceType::Science),
            max_level: 1,
            ..Default::default()
        });

        registry.register(Upgrade {
            id: UpgradeId("science_boost"),
            name: "Research Lab".to_string(),
            description: "Science production +100%".to_string(),
            cost: vec![(ResourceType::Science, 50.0), (ResourceType::Gold, 1000.0)],
            effect: UpgradeEffect::MultiplyResourceRate(ResourceType::Science, 2.0),
            max_level: 5,
            prerequisites: vec![UpgradeId("science_unlock")],
            ..Default::default()
        });

        // Magic upgrades
        registry.register(Upgrade {
            id: UpgradeId("magic_unlock"),
            name: "Apprentice Mage".to_string(),
            description: "Unlocks magic production".to_string(),
            cost: vec![(ResourceType::Science, 100.0), (ResourceType::Gold, 5000.0)],
            effect: UpgradeEffect::UnlockResource(ResourceType::Magic),
            max_level: 1,
            prerequisites: vec![UpgradeId("science_unlock")],
            ..Default::default()
        });
    }

    fn register_buildings(world: &mut World) {
        let mut registry = world.get_resource_mut::<BuildingRegistry>().unwrap();

        registry.register(Building {
            id: BuildingId("mine"),
            name: "Gold Mine".to_string(),
            description: "Produces gold over time".to_string(),
            base_cost: vec![(ResourceType::Gold, 50.0), (ResourceType::Wood, 20.0)],
            cost_growth: 1.15,
            production: vec![(ResourceType::Gold, 1.0)],
            max_level: 100,
            ..Default::default()
        });

        registry.register(Building {
            id: BuildingId("lumber_mill"),
            name: "Lumber Mill".to_string(),
            description: "Produces wood over time".to_string(),
            base_cost: vec![(ResourceType::Gold, 100.0), (ResourceType::Wood, 50.0)],
            cost_growth: 1.15,
            production: vec![(ResourceType::Wood, 2.0)],
            max_level: 100,
            prerequisites: vec![UpgradeId("wood_clicker")],
            ..Default::default()
        });

        registry.register(Building {
            id: BuildingId("quarry"),
            name: "Quarry".to_string(),
            description: "Produces stone over time".to_string(),
            base_cost: vec![(ResourceType::Gold, 500.0), (ResourceType::Wood, 200.0), (ResourceType::Stone, 50.0)],
            cost_growth: 1.2,
            production: vec![(ResourceType::Stone, 1.0)],
            max_level: 50,
            prerequisites: vec![UpgradeId("gold_multiplier")],
            ..Default::default()
        });

        registry.register(Building {
            id: BuildingId("farm"),
            name: "Farm".to_string(),
            description: "Produces food over time".to_string(),
            base_cost: vec![(ResourceType::Gold, 200.0), (ResourceType::Wood, 100.0)],
            cost_growth: 1.15,
            production: vec![(ResourceType::Food, 5.0)],
            max_level: 100,
            ..Default::default()
        });

        registry.register(Building {
            id: BuildingId("laboratory"),
            name: "Laboratory".to_string(),
            description: "Produces science over time".to_string(),
            base_cost: vec![(ResourceType::Gold, 2000.0), (ResourceType::Science, 100.0)],
            cost_growth: 1.25,
            production: vec![(ResourceType::Science, 2.0)],
            max_level: 50,
            prerequisites: vec![UpgradeId("science_unlock")],
            ..Default::default()
        });

        registry.register(Building {
            id: BuildingId("wizard_tower"),
            name: "Wizard Tower".to_string(),
            description: "Produces magic over time".to_string(),
            base_cost: vec![(ResourceType::Gold, 10000.0), (ResourceType::Magic, 50.0), (ResourceType::Science, 500.0)],
            cost_growth: 1.3,
            production: vec![(ResourceType::Magic, 1.0)],
            max_level: 25,
            prerequisites: vec![UpgradeId("magic_unlock")],
            ..Default::default()
        });
    }

    /// Process a single game tick
    pub fn tick(&mut self, delta_time: f32) {
        self.world.resource_mut::<GameState>().tick_accumulator += delta_time;
        self.schedule.run(&mut self.world);
    }

    /// Save game to file
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.world.get_resource::<GameState>().clone();
        crate::save::save_game(&state, "save.json")?;
        tracing::info!("Game saved");
        Ok(())
    }

    /// Load game from file
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(state) = crate::save::load_game("save.json")? {
            self.world.insert_resource(state);
            tracing::info!("Game loaded");
        }
        Ok(())
    }

    /// Get reference to game state
    pub fn state(&self) -> &GameState {
        self.world.get_resource::<GameState>().unwrap()
    }

    /// Get mutable reference to game state
    pub fn state_mut(&mut self) -> Mut<GameState> {
        self.world.get_resource_mut::<GameState>().unwrap()
    }

    /// Purchase an upgrade
    pub fn purchase_upgrade(&mut self, upgrade_id: &UpgradeId) -> bool {
        let mut state = self.world.get_resource_mut::<GameState>().unwrap();
        let registry = self.world.get_resource::<UpgradeRegistry>().unwrap();

        if let Some(upgrade) = registry.get(upgrade_id) {
            let upgrade_state = state.upgrades.entry(upgrade_id.clone()).or_default();

            if upgrade_state.level >= upgrade.max_level {
                return false;
            }

            // Check prerequisites
            for prereq in &upgrade.prerequisites {
                let prereq_state = state.upgrades.get(prereq).unwrap_or(&UpgradeState::default());
                if prereq_state.level == 0 {
                    return false;
                }
            }

            // Check cost (with scaling)
            let level = upgrade_state.level;
            let cost_multiplier = upgrade.cost_growth.powi(level as i32);
            let can_afford = upgrade.cost.iter().all(|(res, base_cost)| {
                state.get_resource(*res) >= base_cost * cost_multiplier
            });

            if !can_afford {
                return false;
            }

            // Pay cost
            for (res, base_cost) in &upgrade.cost {
                let cost = base_cost * cost_multiplier;
                state.spend_resource(*res, cost);
                state.statistics.total_spent.entry(*res).or_insert(0.0) += cost;
            }

            // Apply effect
            upgrade_state.level += 1;
            upgrade_state.total_levels_purchased += 1;
            state.statistics.upgrades_purchased += 1;

            // Apply upgrade effect immediately
            match &upgrade.effect {
                UpgradeEffect::AddResourceRate(res, amount) => {
                    state.modify_rate(*res, *amount);
                }
                UpgradeEffect::MultiplyResourceRate(res, mult) => {
                    let current = state.get_rate(*res);
                    state.set_rate(*res, current * mult);
                }
                UpgradeEffect::UnlockResource(res) => {
                    if state.get_rate(*res) == 0.0 {
                        state.set_rate(*res, 0.01); // Small base rate
                    }
                }
                UpgradeEffect::AddBuildingProduction(bld, res, amount) => {
                    // Handled by building system
                }
                UpgradeEffect::GlobalMultiplier(mult) => {
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
            }

            true
        } else {
            false
        }
    }

    /// Build a building
    pub fn build_building(&mut self, building_id: &BuildingId) -> bool {
        let mut state = self.world.get_resource_mut::<GameState>().unwrap();
        let registry = self.world.get_resource::<BuildingRegistry>().unwrap();

        if let Some(building) = registry.get(building_id) {
            let building_state = state.buildings.entry(building_id.clone()).or_default();

            if building_state.level >= building.max_level {
                return false;
            }

            // Check prerequisites
            for prereq in &building.prerequisites {
                let prereq_state = state.upgrades.get(prereq).unwrap_or(&UpgradeState::default());
                if prereq_state.level == 0 {
                    return false;
                }
            }

            // Check cost (with scaling)
            let level = building_state.level;
            let cost_multiplier = building.cost_growth.powi(level as i32);
            let can_afford = building.base_cost.iter().all(|(res, base_cost)| {
                state.get_resource(*res) >= base_cost * cost_multiplier
            });

            if !can_afford {
                return false;
            }

            // Pay cost
            for (res, base_cost) in &building.base_cost {
                let cost = base_cost * cost_multiplier;
                state.spend_resource(*res, cost);
                state.statistics.total_spent.entry(*res).or_insert(0.0) += cost;
            }

            // Build
            building_state.level += 1;
            building_state.total_built += 1;
            state.statistics.buildings_built += 1;

            true
        } else {
            false
        }
    }

    /// Perform prestige reset
    pub fn prestige(&mut self) -> bool {
        let mut state = self.world.get_resource_mut::<GameState>().unwrap();

        // Calculate prestige points based on total gold earned
        let total_gold = state.statistics.total_gained.get(&ResourceType::Gold).copied().unwrap_or(0.0);
        let points = (total_gold.sqrt() / 10.0).floor() as u64;

        if points == 0 {
            return false;
        }

        // Store current prestige multipliers
        let old_multipliers = state.prestige.multipliers.clone();

        // Reset resources but keep prestige
        state.resources.clear();
        state.resources.insert(ResourceType::Gold, 0.0);
        state.resources.insert(ResourceType::Wood, 0.0);
        state.resources.insert(ResourceType::Stone, 0.0);
        state.resources.insert(ResourceType::Food, 0.0);
        state.resources.insert(ResourceType::Science, 0.0);
        state.resources.insert(ResourceType::Magic, 0.0);

        // Reset rates to base
        state.resource_rates.clear();
        state.resource_rates.insert(ResourceType::Gold, 0.1);
        state.resource_rates.insert(ResourceType::Wood, 0.05);
        state.resource_rates.insert(ResourceType::Stone, 0.02);
        state.resource_rates.insert(ResourceType::Food, 0.0);
        state.resource_rates.insert(ResourceType::Science, 0.0);
        state.resource_rates.insert(ResourceType::Magic, 0.0);

        // Reset upgrades and buildings
        state.upgrades.clear();
        state.buildings.clear();

        // Apply prestige
        state.prestige.level += 1;
        state.prestige.points += points as f64;
        state.prestige.total_points_earned += points as f64;
        state.statistics.prestiges += 1;

        // Add prestige multipliers (each prestige gives +10% to all production)
        let prestige_mult = 1.0 + (state.prestige.level as f64 * 0.1);
        for res in [
            ResourceType::Gold,
            ResourceType::Wood,
            ResourceType::Stone,
            ResourceType::Food,
            ResourceType::Science,
            ResourceType::Magic,
        ] {
            let base = state.prestige.multipliers.get(&res).copied().unwrap_or(1.0);
            state.prestige.multipliers.insert(res, base * prestige_mult);
        }

        // Re-apply prestige multipliers to rates
        for (res, mult) in &state.prestige.multipliers {
            let current = state.get_rate(*res);
            state.set_rate(*res, current * mult);
        }

        true
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
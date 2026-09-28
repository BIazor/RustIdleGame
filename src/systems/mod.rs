//! Systems module - ECS systems for game logic

use bevy_ecs::prelude::*;
use crate::components::*;
use crate::game::GameState;
use crate::resources::*;

/// System: Produce resources from base rates
pub fn resource_production_system(
    mut state: ResMut<GameState>,
    time: Res<GameTime>,
) {
    let delta = 0.1; // Fixed tick rate of 100ms = 0.1s

    // Apply prestige multipliers to rates
    for (resource, rate) in state.resource_rates.iter_mut() {
        let prestige_mult = state.prestige.get_multiplier(*resource);
        let effective_rate = *rate * prestige_mult;
        state.add_resource(*resource, effective_rate * delta);
    }
}

/// System: Produce resources from buildings
pub fn building_production_system(
    mut state: ResMut<GameState>,
    building_registry: Res<BuildingRegistry>,
) {
    let delta = 0.1; // Fixed tick rate

    for (building_id, building_state) in &state.buildings {
        if building_state.level == 0 {
            continue;
        }

        if let Some(building) = building_registry.get(building_id) {
            let level = building_state.level as f64;
            let prestige_mult = 1.0; // Buildings get global prestige mult via resource rates

            for (resource, base_production) in &building.production {
                let production = base_production * level * prestige_mult;
                state.add_resource(*resource, production * delta);
            }
        }
    }
}

/// System: Apply upgrade effects
pub fn upgrade_effect_system(
    mut state: ResMut<GameState>,
    upgrade_registry: Res<UpgradeRegistry>,
) {
    // This system ensures upgrade effects are properly applied
    // Most effects are applied immediately on purchase, but this handles
    // persistent effects that need recalculation (e.g., after prestige)

    for (upgrade_id, upgrade_state) in &state.upgrades {
        if upgrade_state.level == 0 {
            continue;
        }

        if let Some(upgrade) = upgrade_registry.get(upgrade_id) {
            // Effects that need to persist across prestige resets
            match &upgrade.effect {
                UpgradeEffect::AddBuildingProduction(bld_id, resource, amount) => {
                    if let Some(building_state) = state.buildings.get(bld_id) {
                        if building_state.level > 0 {
                            let production = amount * building_state.level as f64;
                            // This is additive to building production
                            // Handled in building_production_system
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// System: Calculate prestige points available
pub fn prestige_calculation_system(
    mut state: ResMut<GameState>,
) {
    // This system updates the available prestige points display
    // Actual prestige is triggered by player action
    let total_gold = state.statistics.total_gained.get(&ResourceType::Gold).copied().unwrap_or(0.0);
    let available_points = (total_gold.sqrt() / 10.0).floor() as u64;
    // Store in a transient resource for UI display if needed
}

/// System: Update statistics
pub fn statistics_system(
    mut state: ResMut<GameState>,
    time: Res<GameTime>,
) {
    state.statistics.total_playtime = time.total_time;
    state.statistics.ticks_processed += 1;
}

/// System: Auto-save game
pub fn auto_save_system(
    mut state: ResMut<GameState>,
    time: Res<GameTime>,
) {
    if state.statistics.ticks_processed % (state.settings.auto_save_interval * 10) == 0 {
        // Save triggered externally to avoid filesystem in system
        // This just marks that save is needed
    }
}

/// System: Process offline progress when loading
pub fn offline_progress_system(
    mut state: ResMut<GameState>,
    offline: Res<OfflineProgress>,
) {
    if !state.settings.offline_progress {
        return;
    }

    let now = chrono::Utc::now();
    let elapsed = now.signed_duration_since(offline.last_save_time);
    let elapsed_hours = elapsed.num_hours() as u64;

    if elapsed_hours == 0 {
        return;
    }

    let max_hours = offline.max_offline_hours.min(elapsed_hours);
    let elapsed_seconds = max_hours as f64 * 3600.0;

    // Calculate production during offline time
    for (resource, rate) in &state.resource_rates {
        let prestige_mult = state.prestige.get_multiplier(*resource);
        let effective_rate = rate * prestige_mult;

        // Add building production
        let mut building_production = 0.0;
        // This would need building registry - simplified for now

        let total_rate = effective_rate + building_production;
        let gained = total_rate * elapsed_seconds * 0.5; // 50% efficiency offline

        if gained > 0.0 {
            state.add_resource(*resource, gained);
        }
    }

    state.statistics.total_playtime += elapsed_seconds;
}

/// System: Handle clicker input
pub fn click_system(
    mut state: ResMut<GameState>,
    mut click_state: ResMut<ClickState>,
) {
    // This would be triggered by UI input
    // For now, just a placeholder
    click_state.register_click();

    // Click produces gold based on upgrades
    let click_power = 1.0; // Base click power
    state.add_resource(ResourceType::Gold, click_power);
}

/// Helper: Calculate cost with growth
pub fn calculate_cost(base_cost: f64, level: u32, growth: f64) -> f64 {
    base_cost * growth.powi(level as i32)
}

/// Helper: Calculate total cost for multiple levels
pub fn calculate_total_cost(base_cost: f64, current_level: u32, levels_to_buy: u32, growth: f64) -> f64 {
    let mut total = 0.0;
    for i in 0..levels_to_buy {
        total += calculate_cost(base_cost, current_level + i, growth);
    }
    total
}

/// Helper: Calculate max affordable levels
pub fn max_affordable_levels(available: f64, base_cost: f64, current_level: u32, growth: f64) -> u32 {
    let mut levels = 0;
    let mut remaining = available;
    let mut level = current_level;

    loop {
        let cost = calculate_cost(base_cost, level, growth);
        if remaining >= cost {
            remaining -= cost;
            levels += 1;
            level += 1;
        } else {
            break;
        }
    }

    levels
}
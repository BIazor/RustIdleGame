//! Save/Load module - Game persistence

use std::fs;
use std::path::Path;
use chrono::Utc;

use crate::game::GameState;
use crate::components::OfflineProgress;

/// Save game to JSON file
pub fn save_game(state: &GameState, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Add offline progress timestamp
    let mut save_data = SaveData {
        game_state: state.clone(),
        offline_progress: OfflineProgress {
            last_save_time: Utc::now(),
            max_offline_hours: state.settings.max_offline_hours,
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        saved_at: Utc::now(),
    };

    let json = serde_json::to_string_pretty(&save_data)?;
    fs::write(path, json)?;

    tracing::info!("Game saved to {}", path);
    Ok(())
}

/// Load game from JSON file
pub fn load_game(path: &str) -> Result<Option<GameState>, Box<dyn std::error::Error>> {
    let path = Path::new(path);

    if !path.exists() {
        tracing::info!("No save file found at {}", path.display());
        return Ok(None);
    }

    let json = fs::read_to_string(path)?;
    let save_data: SaveData = serde_json::from_str(&json)?;

    tracing::info!("Game loaded from {} (saved at {})", path.display(), save_data.saved_at);

    // Handle version migration if needed
    let mut state = migrate_save(save_data.game_state, &save_data.version)?;

    // Update offline progress timestamp
    state.settings.max_offline_hours = save_data.offline_progress.max_offline_hours;

    Ok(Some(state))
}

/// Save data wrapper with metadata
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct SaveData {
    game_state: GameState,
    offline_progress: OfflineProgress,
    version: String,
    saved_at: chrono::DateTime<Utc>,
}

/// Migrate save data from older versions
fn migrate_save(mut state: GameState, from_version: &str) -> Result<GameState, Box<dyn std::error::Error>> {
    let current_version = env!("CARGO_PKG_VERSION");

    if from_version == current_version {
        return Ok(state);
    }

    tracing::info!("Migrating save from version {} to {}", from_version, current_version);

    // Version migration logic would go here
    // For now, just ensure all resource types exist
    use crate::components::ResourceType;
    for res in ResourceType::all() {
        state.resources.entry(res).or_insert(0.0);
        state.resource_rates.entry(res).or_insert(0.0);
    }

    // Ensure prestige multipliers exist
    for res in ResourceType::all() {
        state.prestige.multipliers.entry(res).or_insert(1.0);
    }

    // Ensure statistics entries exist
    for res in ResourceType::all() {
        state.statistics.total_gained.entry(res).or_insert(0.0);
        state.statistics.total_spent.entry(res).or_insert(0.0);
    }

    Ok(state)
}

/// Export save as base64 string (for sharing)
pub fn export_save(state: &GameState) -> Result<String, Box<dyn std::error::Error>> {
    let save_data = SaveData {
        game_state: state.clone(),
        offline_progress: OfflineProgress {
            last_save_time: Utc::now(),
            max_offline_hours: state.settings.max_offline_hours,
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        saved_at: Utc::now(),
    };

    let json = serde_json::to_string(&save_data)?;
    let encoded = base64::encode(json);
    Ok(encoded)
}

/// Import save from base64 string
pub fn import_save(encoded: &str) -> Result<GameState, Box<dyn std::error::Error>> {
    let json = base64::decode(encoded)?;
    let save_data: SaveData = serde_json::from_slice(&json)?;
    let mut state = migrate_save(save_data.game_state, &save_data.version)?;
    state.settings.max_offline_hours = save_data.offline_progress.max_offline_hours;
    Ok(state)
}

/// Delete save file
pub fn delete_save(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path);
    if path.exists() {
        fs::remove_file(path)?;
        tracing::info!("Save file deleted: {}", path.display());
    }
    Ok(())
}

/// List all save files in directory
pub fn list_saves(dir: &str) -> Result<Vec<SaveInfo>, Box<dyn std::error::Error>> {
    let mut saves = Vec::new();
    let path = Path::new(dir);

    if !path.exists() {
        return Ok(saves);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(save_data) = serde_json::from_str::<SaveData>(&json) {
                    saves.push(SaveInfo {
                        path: path.to_string_lossy().to_string(),
                        version: save_data.version,
                        saved_at: save_data.saved_at,
                        prestige_level: save_data.game_state.prestige.level,
                        total_gold: save_data.game_state.statistics.total_gained.get(&crate::components::ResourceType::Gold).copied().unwrap_or(0.0),
                    });
                }
            }
        }
    }

    saves.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
    Ok(saves)
}

#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub path: String,
    pub version: String,
    pub saved_at: chrono::DateTime<Utc>,
    pub prestige_level: u32,
    pub total_gold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameState;
    use crate::components::ResourceType;

    #[test]
    fn test_save_load() {
        let mut state = GameState::default();
        state.add_resource(ResourceType::Gold, 1000.0);
        state.prestige.level = 5;

        let path = "test_save.json";
        save_game(&state, path).unwrap();

        let loaded = load_game(path).unwrap().unwrap();
        assert_eq!(loaded.get_resource(ResourceType::Gold), 1000.0);
        assert_eq!(loaded.prestige.level, 5);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_export_import() {
        let mut state = GameState::default();
        state.add_resource(ResourceType::Gold, 500.0);
        state.add_resource(ResourceType::Wood, 100.0);

        let exported = export_save(&state).unwrap();
        let imported = import_save(&exported).unwrap();

        assert_eq!(imported.get_resource(ResourceType::Gold), 500.0);
        assert_eq!(imported.get_resource(ResourceType::Wood), 100.0);
    }
}
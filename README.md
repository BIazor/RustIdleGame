# Rust Idle Game

A clean, extensible base template for building idle/incremental games in Rust.

## Features

- **ECS Architecture** - Built on Bevy ECS for clean separation of concerns
- **Resource System** - Multiple resource types with production rates
- **Upgrade System** - Purchasable upgrades with prerequisites and scaling costs
- **Building System** - Construct buildings for passive resource generation
- **Prestige System** - Reset progress for permanent multipliers
- **Save/Load** - JSON persistence with version migration support
- **Offline Progress** - Calculate gains while away
- **Terminal UI** - Full-featured TUI using ratatui
- **Statistics Tracking** - Detailed game statistics
- **Achievement Framework** - Extensible achievement system

## Quick Start

```bash
# Clone the repository
git clone https://github.com/BIazor/RustIdleGame.git
cd RustIdleGame

# Build and run
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch panels |
| `↑` / `↓` | Navigate lists |
| `Enter` | Purchase upgrade / Build building / Confirm prestige |
| `Q` / `Esc` | Quit game |
| `S` | Save game |
| `L` | Load game |
| `H` | Toggle help |

## Panels

1. **Resources** - View all resources, amounts, and production rates
2. **Upgrades** - Purchase upgrades to boost production
3. **Buildings** - Construct buildings for passive income
4. **Prestige** - Reset for permanent bonuses
5. **Statistics** - View detailed game statistics
6. **Settings** - Configure game options

## Architecture

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library exports
├── game/
│   └── mod.rs        # Core game logic & state
├── components/
│   └── mod.rs        # ECS components & definitions
├── resources/
│   └── mod.rs        # ECS resources (time, config, achievements)
├── systems/
│   └── mod.rs        # ECS systems (production, upgrades, etc.)
├── ui/
│   └── mod.rs        # Terminal UI (ratatui)
└── save/
    └── mod.rs        # Save/load & export/import
```

## Extending the Game

### Adding a New Resource

1. Add to `ResourceType` enum in `components/mod.rs`
2. Add display name, symbol, and color
3. Add starting values in `GameState::default()`
4. Add to `ResourceType::all()` array

### Adding a New Upgrade

```rust
registry.register(Upgrade {
    id: UpgradeId("my_upgrade"),
    name: "My Upgrade".to_string(),
    description: "Does something cool".to_string(),
    cost: vec![(ResourceType::Gold, 100.0)],
    cost_growth: 1.15,
    effect: UpgradeEffect::MultiplyResourceRate(ResourceType::Gold, 1.5),
    max_level: 10,
    prerequisites: vec![UpgradeId("required_upgrade")],
    ..Default::default()
});
```

### Adding a New Building

```rust
registry.register(Building {
    id: BuildingId("my_building"),
    name: "My Building".to_string(),
    description: "Produces resources".to_string(),
    base_cost: vec![(ResourceType::Gold, 500.0), (ResourceType::Wood, 200.0)],
    cost_growth: 1.15,
    production: vec![(ResourceType::Gold, 10.0)],
    max_level: 100,
    prerequisites: vec![UpgradeId("required_upgrade")],
    ..Default::default()
});
```

### Adding Achievements

```rust
achievements.definitions.push(AchievementDef {
    id: "gold_hoarder".to_string(),
    name: "Gold Hoarder".to_string(),
    description: "Accumulate 1,000,000 gold".to_string(),
    condition: AchievementCondition::ResourceTotal(ResourceType::Gold, 1_000_000.0),
    reward: AchievementReward::GlobalMultiplier(1.1),
    hidden: false,
});
```

## Save Format

Saves are stored as JSON with metadata:

```json
{
  "game_state": { ... },
  "offline_progress": {
    "last_save_time": "2024-01-15T10:30:00Z",
    "max_offline_hours": 24
  },
  "version": "0.1.0",
  "saved_at": "2024-01-15T10:30:00Z"
}
```

### Export/Import

```rust
// Export to base64 string (for sharing)
let encoded = export_save(&state)?;

// Import from base64 string
let state = import_save(&encoded)?;
```

## Configuration

Game settings can be modified in `GameSettings`:

```rust
pub struct GameSettings {
    pub auto_save_interval: u64,      // seconds
    pub show_numbers_scientific: bool,
    pub tick_rate: u64,               // ms
    pub offline_progress: bool,
    pub max_offline_hours: u64,
}
```

## Prestige Formula

Prestige points = `floor(sqrt(total_gold_earned) / 10)`

Each prestige level grants +10% global production multiplier.

## Building Cost Formula

`cost = base_cost × growth_factor^level`

Default growth factor: 1.15x per level

## Upgrade Cost Formula

Same as buildings: `cost = base_cost × growth_factor^level`

## Development

### Running Tests

```bash
cargo test
```

### Building for Release

```bash
cargo build --release
```

### Code Style

```bash
cargo fmt
cargo clippy
```

## License

MIT OR Apache-2.0

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and clippy
5. Submit a PR

## Roadmap

- [ ] WebAssembly support for browser play
- [ ] Graphical UI (egui or bevy)
- [ ] Modding support via Lua/Rhai
- [ ] Multiplayer/leaderboards
- [ ] More content (resources, upgrades, buildings)
- [ ] Achievements UI panel
- [ ] Import/export save codes
- [ ] Cloud save synchronization
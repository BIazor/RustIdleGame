//! UI module - Terminal user interface using ratatui

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

use crate::components::{Panel, UiState, ConfirmationDialog, ConfirmationAction};
use crate::game::GameState;
use crate::resources::format_number;

/// Main UI struct
pub struct Ui {
    pub state: UiState,
    resource_list_state: ListState,
    upgrade_list_state: ListState,
    building_list_state: ListState,
}

impl Ui {
    pub fn new() -> Self {
        let mut resource_list_state = ListState::default();
        resource_list_state.select(Some(0));
        let mut upgrade_list_state = ListState::default();
        upgrade_list_state.select(Some(0));
        let mut building_list_state = ListState::default();
        building_list_state.select(Some(0));

        Self {
            state: UiState::default(),
            resource_list_state,
            upgrade_list_state,
            building_list_state,
        }
    }

    pub fn next_panel(&mut self) {
        self.state.current_panel = match self.state.current_panel {
            Panel::Resources => Panel::Upgrades,
            Panel::Upgrades => Panel::Buildings,
            Panel::Buildings => Panel::Prestige,
            Panel::Prestige => Panel::Statistics,
            Panel::Statistics => Panel::Settings,
            Panel::Settings => Panel::Resources,
        };
        self.state.selected_index = 0;
        self.update_list_states();
    }

    pub fn prev_panel(&mut self) {
        self.state.current_panel = match self.state.current_panel {
            Panel::Resources => Panel::Settings,
            Panel::Upgrades => Panel::Resources,
            Panel::Buildings => Panel::Upgrades,
            Panel::Prestige => Panel::Buildings,
            Panel::Statistics => Panel::Prestige,
            Panel::Settings => Panel::Statistics,
        };
        self.state.selected_index = 0;
        self.update_list_states();
    }

    fn update_list_states(&mut self) {
        match self.state.current_panel {
            Panel::Resources => {
                self.resource_list_state.select(Some(self.state.selected_index));
            }
            Panel::Upgrades => {
                self.upgrade_list_state.select(Some(self.state.selected_index));
            }
            Panel::Buildings => {
                self.building_list_state.select(Some(self.state.selected_index));
            }
            _ => {}
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let max = match self.state.current_panel {
            Panel::Resources => 6, // 6 resource types
            Panel::Upgrades => self.get_available_upgrades_count(),
            Panel::Buildings => self.get_available_buildings_count(),
            Panel::Prestige => 2, // Prestige button + cancel
            Panel::Statistics => 0,
            Panel::Settings => 5,
        };

        if max == 0 {
            return;
        }

        self.state.selected_index = ((self.state.selected_index as i32 + delta).clamp(0, max - 1)) as usize;
        self.update_list_states();
    }

    fn get_available_upgrades_count(&self) -> usize {
        // This would need access to game state - simplified
        10
    }

    fn get_available_buildings_count(&self) -> usize {
        10
    }

    pub fn confirm_selection(&mut self, game: &mut crate::game::Game) {
        match self.state.current_panel {
            Panel::Upgrades => {
                let upgrades = self.get_available_upgrades(game);
                if let Some(upgrade) = upgrades.get(self.state.selected_index) {
                    game.purchase_upgrade(&upgrade.id);
                }
            }
            Panel::Buildings => {
                let buildings = self.get_available_buildings(game);
                if let Some(building) = buildings.get(self.state.selected_index) {
                    game.build_building(&building.id);
                }
            }
            Panel::Prestige => {
                if self.state.selected_index == 0 {
                    self.state.confirmation_dialog = Some(ConfirmationDialog {
                        title: "Prestige".to_string(),
                        message: format!(
                            "Reset progress for {:.0} prestige points?\nThis will reset all resources, upgrades, and buildings.",
                            self.calculate_prestige_points(game)
                        ),
                        on_confirm: ConfirmationAction::Prestige,
                    });
                }
            }
            Panel::Settings => {
                // Handle settings changes
            }
            _ => {}
        }
    }

    fn get_available_upgrades(&self, game: &crate::game::Game) -> Vec<crate::components::Upgrade> {
        let state = game.state();
        let registry = game.world.get_resource::<crate::components::UpgradeRegistry>().unwrap();
        registry.available(state)
            .into_iter()
            .cloned()
            .collect()
    }

    fn get_available_buildings(&self, game: &crate::game::Game) -> Vec<crate::components::Building> {
        let state = game.state();
        let registry = game.world.get_resource::<crate::components::BuildingRegistry>().unwrap();
        registry.available(state)
            .into_iter()
            .cloned()
            .collect()
    }

    fn calculate_prestige_points(&self, game: &crate::game::Game) -> f64 {
        let state = game.state();
        let total_gold = state.statistics.total_gained.get(&crate::components::ResourceType::Gold).copied().unwrap_or(0.0);
        (total_gold.sqrt() / 10.0).floor()
    }

    pub fn render(&mut self, frame: &mut Frame, game: &crate::game::Game) {
        let state = game.state();
        let area = frame.size();

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Main content
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        self.render_header(frame, chunks[0], game);

        // Main content area split into sidebar and main panel
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // Sidebar (panel tabs)
                Constraint::Min(40),    // Main panel content
            ])
            .split(chunks[1]);

        self.render_sidebar(frame, main_chunks[0], game);
        self.render_main_panel(frame, main_chunks[1], game);

        // Footer
        self.render_footer(frame, chunks[2], game);

        // Confirmation dialog overlay
        if let Some(dialog) = &self.state.confirmation_dialog {
            self.render_confirmation_dialog(frame, area, dialog);
        }

        // Help overlay
        if self.state.show_help {
            self.render_help(frame, area);
        }
    }

    fn render_header(&self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();
        let prestige_text = if state.prestige.level > 0 {
            format!(" [P{}]", state.prestige.level)
        } else {
            String::new()
        };

        let header = Paragraph::new(Line::from(vec![
            Span::styled("⚙ Rust Idle Game", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(prestige_text, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::DarkGray)),
        ]))
        .block(Block::default().borders(Borders::ALL).title(" Game "));

        frame.render_widget(header, area);
    }

    fn render_sidebar(&mut self, frame: &mut Frame, area: Rect, _game: &crate::game::Game) {
        let panels = [
            (Panel::Resources, "📊 Resources", "1"),
            (Panel::Upgrades, "⬆ Upgrades", "2"),
            (Panel::Buildings, "🏗 Buildings", "3"),
            (Panel::Prestige, "✨ Prestige", "4"),
            (Panel::Statistics, "📈 Statistics", "5"),
            (Panel::Settings, "⚙ Settings", "6"),
        ];

        let items: Vec<ListItem> = panels.iter().map(|(panel, name, key)| {
            let selected = *panel == self.state.current_panel;
            let style = if selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", key), Style::default().fg(Color::DarkGray)),
                Span::styled(name, style),
            ]))
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Panels "))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.resource_list_state);
    }

    fn render_main_panel(&mut self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        match self.state.current_panel {
            Panel::Resources => self.render_resources_panel(frame, area, game),
            Panel::Upgrades => self.render_upgrades_panel(frame, area, game),
            Panel::Buildings => self.render_buildings_panel(frame, area, game),
            Panel::Prestige => self.render_prestige_panel(frame, area, game),
            Panel::Statistics => self.render_statistics_panel(frame, area, game),
            Panel::Settings => self.render_settings_panel(frame, area, game),
        }
    }

    fn render_resources_panel(&self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();
        let scientific = state.settings.show_numbers_scientific;

        let resources = [
            ResourceType::Gold,
            ResourceType::Wood,
            ResourceType::Stone,
            ResourceType::Food,
            ResourceType::Science,
            ResourceType::Magic,
        ];

        let lines: Vec<Line> = resources.iter().map(|res| {
            let amount = state.get_resource(*res);
            let rate = state.get_rate(*res);
            let prestige_mult = state.prestige.get_multiplier(*res);
            let effective_rate = rate * prestige_mult;

            Line::from(vec![
                Span::styled(format!("{} ", res.symbol()), Style::default().fg(res.color())),
                Span::styled(format!("{:<10}", res.display_name()), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>15}", format_number(amount, scientific)), Style::default().fg(res.color())),
                Span::styled("  ", Style::default()),
                Span::styled(format!("{:+.2}/s", effective_rate), Style::default().fg(Color::Green)),
                if prestige_mult > 1.0 {
                    Span::styled(format!(" (×{:.1})", prestige_mult), Style::default().fg(Color::Magenta))
                } else {
                    Span::raw("")
                },
            ])
        }).collect();

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Resources "))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_upgrades_panel(&mut self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();
        let registry = game.world.get_resource::<crate::components::UpgradeRegistry>().unwrap();
        let scientific = state.settings.show_numbers_scientific;

        let available: Vec<_> = registry.available(state).cloned().collect();

        if available.is_empty() {
            let paragraph = Paragraph::new("No upgrades available yet.\nProgress to unlock more!")
                .block(Block::default().borders(Borders::ALL).title(" Upgrades "))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = available.iter().enumerate().map(|(i, upgrade)| {
            let upgrade_state = state.upgrades.get(&upgrade.id).unwrap_or(&crate::components::UpgradeState::default());
            let level = upgrade_state.level;
            let max_level = upgrade.max_level;
            let can_afford = self.can_afford_upgrade(state, upgrade, level);

            let cost_text = if level < max_level {
                let cost_mult = upgrade.cost_growth.powi(level as i32);
                upgrade.cost.iter().map(|(res, base)| {
                    let cost = base * cost_mult;
                    format!("{} {}", res.symbol(), format_number(cost, scientific))
                }).collect::<Vec<_>>().join(", ")
            } else {
                "MAX".to_string()
            };

            let style = if level >= max_level {
                Style::default().fg(Color::DarkGray)
            } else if can_afford {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let selected = i == self.state.selected_index;
            let prefix = if selected { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} ", upgrade.name), style.add_modifier(Modifier::BOLD)),
                Span::styled(format!("Lv.{}/{} ", level, max_level), style),
                Span::styled(cost_text, Style::default().fg(Color::Yellow)),
            ]))
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Upgrades (Enter to buy) "))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        frame.render_stateful_widget(list, area, &mut self.upgrade_list_state);
    }

    fn can_afford_upgrade(&self, state: &GameState, upgrade: &crate::components::Upgrade, level: u32) -> bool {
        if level >= upgrade.max_level {
            return false;
        }
        let cost_mult = upgrade.cost_growth.powi(level as i32);
        upgrade.cost.iter().all(|(res, base)| {
            state.get_resource(*res) >= base * cost_mult
        })
    }

    fn render_buildings_panel(&mut self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();
        let registry = game.world.get_resource::<crate::components::BuildingRegistry>().unwrap();
        let scientific = state.settings.show_numbers_scientific;

        let available: Vec<_> = registry.available(state).cloned().collect();

        if available.is_empty() {
            let paragraph = Paragraph::new("No buildings available yet.\nUnlock upgrades to access buildings!")
                .block(Block::default().borders(Borders::ALL).title(" Buildings "))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = available.iter().enumerate().map(|(i, building)| {
            let building_state = state.buildings.get(&building.id).unwrap_or(&crate::components::BuildingState::default());
            let level = building_state.level;
            let max_level = building.max_level;
            let can_afford = self.can_afford_building(state, building, level);

            let cost_text = if level < max_level {
                let cost_mult = building.cost_growth.powi(level as i32);
                building.base_cost.iter().map(|(res, base)| {
                    let cost = base * cost_mult;
                    format!("{} {}", res.symbol(), format_number(cost, scientific))
                }).collect::<Vec<_>>().join(", ")
            } else {
                "MAX".to_string()
            };

            let production_text = building.production.iter().map(|(res, base)| {
                let prod = base * level as f64;
                format!("{} {:.1}/s", res.symbol(), prod)
            }).collect::<Vec<_>>().join(", ");

            let style = if level >= max_level {
                Style::default().fg(Color::DarkGray)
            } else if can_afford {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let selected = i == self.state.selected_index;
            let prefix = if selected { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} ", building.name), style.add_modifier(Modifier::BOLD)),
                Span::styled(format!("Lv.{}/{} ", level, max_level), style),
                Span::styled(cost_text, Style::default().fg(Color::Yellow)),
                Span::styled(" → ", Style::default().fg(Color::DarkGray)),
                Span::styled(production_text, Style::default().fg(Color::Cyan)),
            ]))
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Buildings (Enter to build) "))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        frame.render_stateful_widget(list, area, &mut self.building_list_state);
    }

    fn can_afford_building(&self, state: &GameState, building: &crate::components::Building, level: u32) -> bool {
        if level >= building.max_level {
            return false;
        }
        let cost_mult = building.cost_growth.powi(level as i32);
        building.base_cost.iter().all(|(res, base)| {
            state.get_resource(*res) >= base * cost_mult
        })
    }

    fn render_prestige_panel(&self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();
        let scientific = state.settings.show_numbers_scientific;

        let total_gold = state.statistics.total_gained.get(&crate::components::ResourceType::Gold).copied().unwrap_or(0.0);
        let available_points = (total_gold.sqrt() / 10.0).floor() as u64;

        let lines = vec![
            Line::from(vec![
                Span::styled("Prestige Level: ", Style::default().fg(Color::White)),
                Span::styled(state.prestige.level.to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Total Prestige Points: ", Style::default().fg(Color::White)),
                Span::styled(state.prestige.total_points_earned.to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Current Points: ", Style::default().fg(Color::White)),
                Span::styled(state.prestige.points.to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Total Gold Earned: ", Style::default().fg(Color::White)),
                Span::styled(format_number(total_gold, scientific), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Available Prestige Points: ", Style::default().fg(Color::White)),
                Span::styled(available_points.to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("Prestige Bonus Multipliers:"),
        ];

        let mut mult_lines: Vec<Line> = state.prestige.multipliers.iter().map(|(res, mult)| {
            Line::from(vec![
                Span::styled(format!("  {} ", res.symbol()), Style::default().fg(res.color())),
                Span::styled(format!("{:<10} ×{:.2}", res.display_name(), mult), Style::default().fg(Color::White)),
            ])
        }).collect();

        let mut all_lines = lines;
        all_lines.append(&mut mult_lines);

        all_lines.push(Line::from(""));
        if available_points > 0 {
            all_lines.push(Line::from(vec![
                Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                Span::styled("PRESTIGE NOW", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            all_lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("PRESTIGE (need more gold)", Style::default().fg(Color::DarkGray)),
            ]));
        }
        all_lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Cancel", Style::default().fg(Color::White)),
        ]));

        let paragraph = Paragraph::new(all_lines)
            .block(Block::default().borders(Borders::ALL).title(" Prestige "))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_statistics_panel(&self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();
        let scientific = state.settings.show_numbers_scientific;

        let lines = vec![
            Line::from(vec![
                Span::styled("Play Time: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.1}h", state.statistics.total_playtime / 3600.0), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Ticks Processed: ", Style::default().fg(Color::White)),
                Span::styled(state.statistics.ticks_processed.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Upgrades Purchased: ", Style::default().fg(Color::White)),
                Span::styled(state.statistics.upgrades_purchased.to_string(), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Buildings Built: ", Style::default().fg(Color::White)),
                Span::styled(state.statistics.buildings_built.to_string(), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Prestiges: ", Style::default().fg(Color::White)),
                Span::styled(state.statistics.prestiges.to_string(), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from("Total Resources Gained:"),
        ];

        let mut gained_lines: Vec<Line> = ResourceType::all().iter().map(|res| {
            let gained = state.statistics.total_gained.get(res).copied().unwrap_or(0.0);
            Line::from(vec![
                Span::styled(format!("  {} ", res.symbol()), Style::default().fg(res.color())),
                Span::styled(format!("{:<10}", res.display_name()), Style::default().fg(Color::White)),
                Span::styled(format_number(gained, scientific), Style::default().fg(res.color())),
            ])
        }).collect();

        let mut spent_lines = vec![Line::from(""), Line::from("Total Resources Spent:")];
        let mut spent_items: Vec<Line> = ResourceType::all().iter().map(|res| {
            let spent = state.statistics.total_spent.get(res).copied().unwrap_or(0.0);
            Line::from(vec![
                Span::styled(format!("  {} ", res.symbol()), Style::default().fg(res.color())),
                Span::styled(format!("{:<10}", res.display_name()), Style::default().fg(Color::White)),
                Span::styled(format_number(spent, scientific), Style::default().fg(res.color())),
            ])
        }).collect();

        let mut all_lines = lines;
        all_lines.append(&mut gained_lines);
        all_lines.append(&mut spent_lines);
        all_lines.append(&mut spent_items);

        let paragraph = Paragraph::new(all_lines)
            .block(Block::default().borders(Borders::ALL).title(" Statistics "))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_settings_panel(&self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();

        let lines = vec![
            Line::from(vec![
                Span::styled("Auto-save Interval: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}s", state.settings.auto_save_interval), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Scientific Notation: ", Style::default().fg(Color::White)),
                Span::styled(if state.settings.show_numbers_scientific { "ON" } else { "OFF" }, Style::default().fg(if state.settings.show_numbers_scientific { Color::Green } else { Color::Red })),
            ]),
            Line::from(vec![
                Span::styled("Tick Rate: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}ms", state.settings.tick_rate), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Offline Progress: ", Style::default().fg(Color::White)),
                Span::styled(if state.settings.offline_progress { "ON" } else { "OFF" }, Style::default().fg(if state.settings.offline_progress { Color::Green } else { Color::Red })),
            ]),
            Line::from(vec![
                Span::styled("Max Offline Hours: ", Style::default().fg(Color::White)),
                Span::styled(state.settings.max_offline_hours.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from("Press Enter to toggle (not implemented in demo)"),
        ];

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Settings "))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect, game: &crate::game::Game) {
        let state = game.state();
        let time = game.world.get_resource::<crate::resources::GameTime>().unwrap();

        let help_text = match self.state.current_panel {
            Panel::Upgrades | Panel::Buildings => "Tab/Shift+Tab: Switch panels | ↑/↓: Navigate | Enter: Buy/Build | Q/Esc: Quit | S: Save | L: Load",
            Panel::Prestige => "Tab/Shift+Tab: Switch panels | ↑/↓: Select | Enter: Confirm | Q/Esc: Quit",
            _ => "Tab/Shift+Tab: Switch panels | Q/Esc: Quit | S: Save | L: Load | H: Help",
        };

        let footer = Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", time.format_time()), Style::default().fg(Color::Cyan)),
            Span::styled("|", Style::default().fg(Color::DarkGray)),
            Span::styled(help_text, Style::default().fg(Color::DarkGray)),
        ]))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, area);
    }

    fn render_confirmation_dialog(&self, frame: &mut Frame, area: Rect, dialog: &ConfirmationDialog) {
        let popup_area = centered_rect(60, 40, area);
        frame.render_widget(Clear, popup_area);

        let lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(&dialog.message, Style::default().fg(Color::White))]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [Enter] Confirm  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("  [Esc] Cancel  ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(dialog.title.clone()))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, popup_area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(80, 80, area);
        frame.render_widget(Clear, popup_area);

        let lines = vec![
            Line::from(vec![Span::styled("Rust Idle Game - Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![Span::styled("KEYBINDINGS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  Tab / Shift+Tab    - Switch between panels"),
            Line::from("  ↑ / ↓              - Navigate lists"),
            Line::from("  Enter              - Confirm purchase / build / prestige"),
            Line::from("  Q / Esc            - Quit game"),
            Line::from("  S                  - Save game"),
            Line::from("  L                  - Load game"),
            Line::from("  H                  - Toggle this help"),
            Line::from(""),
            Line::from(vec![Span::styled("PANELS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  Resources  - View resource amounts and production rates"),
            Line::from("  Upgrades   - Purchase upgrades to boost production"),
            Line::from("  Buildings  - Construct buildings for passive income"),
            Line::from("  Prestige   - Reset for permanent bonuses"),
            Line::from("  Statistics - View detailed game statistics"),
            Line::from("  Settings   - Configure game options"),
            Line::from(""),
            Line::from(vec![Span::styled("PRESTIGE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  Prestige resets all progress but grants permanent"),
            Line::from("  production multipliers. Points based on total gold earned."),
            Line::from("  Formula: floor(sqrt(total_gold) / 10)"),
            Line::from(""),
            Line::from(vec![Span::styled("BUILDING COSTS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  Cost = base_cost × growth_factor^level"),
            Line::from("  Default growth: 1.15x per level"),
            Line::from(""),
            Line::from("Press H or Esc to close"),
        ];

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, popup_area);
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
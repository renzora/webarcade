use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod database;
mod events;
mod game;
mod router;

pub use database::*;
pub use events::*;
pub use game::*;

pub struct RoulettePlugin;

#[async_trait]
impl Plugin for RoulettePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "roulette".to_string(),
            name: "Roulette Game".to_string(),
            version: "1.0.0".to_string(),
            description: "European roulette betting game".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec!["currency".to_string()],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Roulette] Initializing plugin...");

        // Database migrations
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS roulette_games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                status TEXT NOT NULL,
                winning_number INTEGER,
                created_at INTEGER NOT NULL,
                spin_started_at INTEGER,
                completed_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS roulette_bets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                amount INTEGER NOT NULL,
                bet_type TEXT NOT NULL,
                bet_value TEXT NOT NULL,
                won BOOLEAN,
                payout INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (game_id) REFERENCES roulette_games(id)
            );

            CREATE TABLE IF NOT EXISTS roulette_config (
                channel TEXT PRIMARY KEY,
                enabled INTEGER NOT NULL DEFAULT 1,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_roulette_games_channel ON roulette_games(channel);
            CREATE INDEX IF NOT EXISTS idx_roulette_games_status ON roulette_games(status);
            CREATE INDEX IF NOT EXISTS idx_roulette_bets_game ON roulette_bets(game_id);
            "#,
            // Add missing columns to existing tables (only won is missing)
            r#"
            ALTER TABLE roulette_bets ADD COLUMN won BOOLEAN;
            "#,
        ])?;

        // Register services
        ctx.provide_service("start_game", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let game_id = database::start_game(&conn, &channel)?;
            Ok(serde_json::json!({ "game_id": game_id }))
        }).await;

        ctx.provide_service("place_bet", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let amount: i64 = serde_json::from_value(input["amount"].clone())?;
            let bet_type: String = serde_json::from_value(input["bet_type"].clone())?;
            let bet_value: String = serde_json::from_value(input["bet_value"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::place_bet(&conn, &channel, &user_id, &username, amount, &bet_type, &bet_value)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("spin_wheel", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let winning_number = database::spin_wheel(&conn, &channel)?;
            Ok(serde_json::json!({ "winning_number": winning_number }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Roulette] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Roulette] Starting plugin...");

        // Subscribe to Twitch chat commands
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("twitch.chat_message").await;

            while let Ok(event) = events.recv().await {
                if let (Ok(channel), Ok(username), Ok(user_id), Ok(message)) = (
                    serde_json::from_value::<String>(event.payload["channel"].clone()),
                    serde_json::from_value::<String>(event.payload["username"].clone()),
                    serde_json::from_value::<String>(event.payload["user_id"].clone()),
                    serde_json::from_value::<String>(event.payload["message"].clone()),
                ) {
                    let parts: Vec<String> = message.split_whitespace().map(|s| s.to_string()).collect();
                    let ctx_cmd = ctx_clone.clone();

                    match parts.get(0).map(|s| s.as_str()) {
                        Some("!roulette") if parts.len() >= 2 => {
                            let args: Vec<String> = parts[1..].to_vec();
                            tokio::spawn(async move {
                                handle_roulette_command(&channel, &args, ctx_cmd).await;
                            });
                        }
                        Some("!bet") | Some("!b") => {
                            let args: Vec<String> = parts[1..].to_vec();
                            tokio::spawn(async move {
                                handle_bet_command(&channel, &username, &user_id, &args, ctx_cmd).await;
                            });
                        }
                        _ => {}
                    }
                }
            }
        });

        log::info!("[Roulette] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Roulette] Stopping plugin...");
        Ok(())
    }
}

async fn handle_roulette_command(channel: &str, args: &[String], ctx: Arc<PluginContext>) {
    if args.is_empty() {
        return;
    }

    let conn_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn_path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("[Roulette] Failed to open database: {}", e);
            return;
        }
    };

    match args[0].to_lowercase().as_str() {
        "on" | "enable" => {
            // Enable roulette overlay and start game
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let channel_str = channel.to_string();
            let result = conn.execute(
                "INSERT OR REPLACE INTO roulette_config (channel, enabled, updated_at) VALUES (?1, 1, ?2)",
                rusqlite::params![&channel_str, now],
            );

            match result {
                Ok(_) => {
                    log::info!("[Roulette] Roulette enabled for {}", channel);

                    // Emit enabled event
                    ctx.emit("roulette.enabled", &serde_json::json!({
                        "channel": channel
                    }));

                    let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                        "channel": channel,
                        "message": "🎰 Roulette has been enabled!"
                    })).await;

                    // Auto-start a game
                    match database::get_active_game(&conn, channel) {
                        Ok(Some(_game)) => {
                            log::info!("[Roulette] Game already in progress for {}", channel);
                        },
                        Ok(None) => {
                            // Start new game
                            if let Ok(game_id) = database::start_game(&conn, channel) {
                                ctx.emit("roulette.game_started", &serde_json::json!({
                                    "channel": channel,
                                    "game_id": game_id,
                                    "timer_seconds": 30
                                }));

                                let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                                    "channel": channel,
                                    "message": "🎰 Roulette table is open! Place your bets with !bet <amount> <type>. Spinning in 30 seconds!"
                                })).await;

                                // Start 30-second auto-spin timer
                                let ctx_timer = ctx.clone();
                                let channel_clone = channel.to_string();
                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                                    let conn_path = crate::core::database::get_database_path();
                                    if let Ok(conn) = rusqlite::Connection::open(&conn_path) {
                                        if let Ok(Some(game)) = database::get_active_game(&conn, &channel_clone) {
                                            if game.status == "betting" {
                                                let _ = database::start_spin(&conn, game.id);
                                                ctx_timer.emit("roulette.spin_started", &serde_json::json!({
                                                    "channel": channel_clone,
                                                    "game_id": game.id
                                                }));
                                            }
                                        }
                                    }
                                });
                            }
                        },
                        Err(e) => {
                            log::error!("[Roulette] Failed to check for active game: {}", e);
                        }
                    }
                },
                Err(e) => {
                    log::error!("[Roulette] Failed to enable roulette: {}", e);
                    let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                        "channel": channel,
                        "message": "❌ Failed to enable roulette"
                    })).await;
                }
            }
        },
        "off" | "disable" => {
            // Disable roulette overlay
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let channel_str = channel.to_string();
            let result = conn.execute(
                "INSERT OR REPLACE INTO roulette_config (channel, enabled, updated_at) VALUES (?1, 0, ?2)",
                rusqlite::params![&channel_str, now],
            );

            match result {
                Ok(_) => {
                    log::info!("[Roulette] Roulette disabled for {}", channel);

                    // Cancel any active game
                    let _ = database::cancel_game(&conn, channel);

                    // Emit disabled event
                    ctx.emit("roulette.disabled", &serde_json::json!({
                        "channel": channel
                    }));

                    let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                        "channel": channel,
                        "message": "🎰 Roulette has been disabled"
                    })).await;
                },
                Err(e) => {
                    log::error!("[Roulette] Failed to disable roulette: {}", e);
                    let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                        "channel": channel,
                        "message": "❌ Failed to disable roulette"
                    })).await;
                }
            }
        },
        "start" => {
            // Check if roulette is enabled
            let enabled: bool = conn.query_row(
                "SELECT enabled FROM roulette_config WHERE channel = ?1",
                [channel],
                |row| row.get(0)
            ).unwrap_or(true); // Default to enabled if not configured

            if !enabled {
                let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                    "channel": channel,
                    "message": "🎰 Roulette is currently disabled. Use !roulette on to enable it."
                })).await;
                return;
            }

            // Check if there's already an active game
            match database::get_active_game(&conn, channel) {
                Ok(Some(_game)) => {
                    log::info!("[Roulette] Game already in progress for {}", channel);
                    let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                        "channel": channel,
                        "message": "🎰 A roulette game is already in progress!"
                    })).await;
                },
                Ok(None) => {
                    // Start new game
                    match database::start_game(&conn, channel) {
                        Ok(game_id) => {
                            // Emit game started event with 30 second timer
                            ctx.emit("roulette.game_started", &serde_json::json!({
                                "channel": channel,
                                "game_id": game_id,
                                "timer_seconds": 30
                            }));

                            // Send chat message
                            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                                "channel": channel,
                                "message": "🎰 Roulette table is open! Place your bets with !bet <amount> <type>. Spinning in 30 seconds!"
                            })).await;

                            // Start 30-second auto-spin timer
                            let ctx_timer = ctx.clone();
                            let channel_clone = channel.to_string();
                            tokio::spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                                // Check if game is still in betting state
                                let conn_path = crate::core::database::get_database_path();
                                if let Ok(conn) = rusqlite::Connection::open(&conn_path) {
                                    if let Ok(Some(game)) = database::get_active_game(&conn, &channel_clone) {
                                        if game.status == "betting" {
                                            // Auto-spin
                                            let _ = database::start_spin(&conn, game.id);
                                            ctx_timer.emit("roulette.spin_started", &serde_json::json!({
                                                "channel": channel_clone,
                                                "game_id": game.id
                                            }));
                                        }
                                    }
                                }
                            });

                            log::info!("[Roulette] Game {} started for {}", game_id, channel);
                        },
                        Err(e) => {
                            log::error!("[Roulette] Failed to start game: {}", e);
                            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                                "channel": channel,
                                "message": "❌ Failed to start roulette game"
                            })).await;
                        }
                    }
                },
                Err(e) => {
                    log::error!("[Roulette] Failed to check for active game: {}", e);
                }
            }
        },
        "spin" => {
            // Manual spin
            match database::get_active_game(&conn, channel) {
                Ok(Some(game)) if game.status == "betting" => {
                    if database::start_spin(&conn, game.id).is_ok() {
                        ctx.emit("roulette.spin_started", &serde_json::json!({
                            "channel": channel,
                            "game_id": game.id
                        }));
                        log::info!("[Roulette] Game {} spinning", game.id);
                    }
                },
                _ => {}
            }
        },
        "refund" | "cancel" => {
            // Refund all bets and cancel game
            match database::get_active_game(&conn, channel) {
                Ok(Some(game)) => {
                    // Get all bets
                    if let Ok(bets) = database::get_game_bets(&conn, game.id) {
                        // Refund each bet via currency plugin
                        for bet in bets {
                            let _ = ctx.call_service("currency", "add_currency", serde_json::json!({
                                "channel": channel,
                                "username": bet.username,
                                "amount": bet.amount,
                                "reason": "Roulette bet refund"
                            })).await;
                        }
                    }

                    // Cancel game
                    let _ = database::cancel_game(&conn, channel);

                    ctx.emit("roulette.game_stopped", &serde_json::json!({
                        "channel": channel,
                        "game_id": game.id
                    }));

                    log::info!("[Roulette] Game {} refunded and cancelled", game.id);
                },
                _ => {}
            }
        },
        "stop" | "close" => {
            // Close game without refund
            match database::get_active_game(&conn, channel) {
                Ok(Some(game)) => {
                    let _ = database::cancel_game(&conn, channel);
                    ctx.emit("roulette.game_stopped", &serde_json::json!({
                        "channel": channel,
                        "game_id": game.id
                    }));
                    log::info!("[Roulette] Game {} stopped", game.id);
                },
                _ => {}
            }
        },
        _ => {}
    }
}

async fn handle_bet_command(channel: &str, username: &str, user_id: &str, args: &[String], ctx: Arc<PluginContext>) {
    if args.is_empty() {
        return;
    }

    // Parse amount
    let amount = match args[0].as_str().parse::<i64>() {
        Ok(amt) if amt >= 10 => amt,
        _ => {
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": format!("@{} ❌ Minimum bet is 10 coins", username)
            })).await;
            return;
        }
    };

    // Get active game
    let conn_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn_path) {
        Ok(c) => c,
        Err(_) => return
    };

    let game = match database::get_active_game(&conn, channel) {
        Ok(Some(g)) if g.status == "betting" => g,
        _ => {
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": format!("@{} 🎰 No active roulette game. Wait for a mod to start one!", username)
            })).await;
            return;
        }
    };

    // Check balance
    let balance_result = ctx.call_service("currency", "get_balance", serde_json::json!({
        "channel": channel,
        "username": username
    })).await;

    let balance = match balance_result {
        Ok(result) => result["balance"].as_i64().unwrap_or(0),
        _ => return
    };

    // Parse bet targets (support multiple bets in one command)
    let mut bet_targets = Vec::new();
    for bet_input in &args[1..] {
        let bet_input_lower = bet_input.as_str().to_lowercase();
        let (bet_type, bet_value) = if let Ok(num) = bet_input_lower.parse::<i64>() {
            if num < 0 || num > 36 {
                continue;
            }
            ("number", num.to_string())
        } else {
            match bet_input_lower.as_str() {
                "red" => ("red", "red".to_string()),
                "black" => ("black", "black".to_string()),
                "odd" => ("odd", "odd".to_string()),
                "even" => ("even", "even".to_string()),
                "low" => ("low", "low".to_string()),
                "high" => ("high", "high".to_string()),
                "dozen1" | "1st" => ("dozen1", "dozen1".to_string()),
                "dozen2" | "2nd" => ("dozen2", "dozen2".to_string()),
                "dozen3" | "3rd" => ("dozen3", "dozen3".to_string()),
                _ => continue
            }
        };
        bet_targets.push((bet_type, bet_value));
    }

    if bet_targets.is_empty() {
        return;
    }

    let total_cost = amount * bet_targets.len() as i64;
    if balance < total_cost {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": format!("@{} ❌ Not enough coins! You need {} but have {}", username, total_cost, balance)
        })).await;
        return;
    }

    // Deduct currency
    let deduct_result = ctx.call_service("currency", "deduct_currency", serde_json::json!({
        "channel": channel,
        "username": username,
        "amount": total_cost,
        "reason": "Roulette bet"
    })).await;

    if deduct_result.is_err() {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": format!("@{} ❌ Failed to place bet (currency error)", username)
        })).await;
        return;
    }

    // Place all bets
    let mut success_count = 0;
    let mut bet_names = Vec::new();
    for (bet_type, bet_value) in bet_targets {
        if database::place_bet(&conn, channel, user_id, username, amount, bet_type, &bet_value).is_ok() {
            ctx.emit("roulette.bet_placed", &serde_json::json!({
                "channel": channel,
                "game_id": game.id,
                "username": username,
                "user_id": user_id,
                "amount": amount,
                "bet_type": bet_type,
                "bet_value": bet_value
            }));
            success_count += 1;
            bet_names.push(bet_value);
        }
    }

    // Send success message
    if success_count > 0 {
        let payout_multiplier = if bet_names.len() == 1 {
            let bet_type = if bet_names[0].chars().all(|c| c.is_numeric()) { "number" } else { bet_names[0].as_str() };
            match bet_type {
                "number" => 35,
                "dozen1" | "dozen2" | "dozen3" => 2,
                _ => 1
            }
        } else {
            0
        };

        let message = if success_count == 1 {
            if payout_multiplier > 0 {
                format!("🎰 @{} bet {} coins on {}! Potential win: {} coins ({}:1)",
                    username, amount, bet_names[0], amount * (payout_multiplier + 1), payout_multiplier)
            } else {
                format!("🎰 @{} bet {} coins on {}!", username, amount, bet_names[0])
            }
        } else {
            format!("🎰 @{} placed {} bets of {} coins each! Total: {} coins",
                username, success_count, amount, total_cost)
        };

        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": message
        })).await;
    }

    log::info!("[Roulette] {} placed {} bet(s) of {} coins", username, success_count, amount);
}

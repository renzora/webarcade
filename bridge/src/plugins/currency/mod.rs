use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod database;
mod events;
mod router;

pub use database::*;
pub use events::*;

pub struct CurrencyPlugin;

#[async_trait]
impl Plugin for CurrencyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "currency".to_string(),
            name: "Currency System".to_string(),
            version: "1.0.0".to_string(),
            description: "User points and currency management".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Currency] Initializing plugin...");

        // Database migrations
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS currency_accounts (
                user_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                balance INTEGER NOT NULL DEFAULT 0,
                lifetime_earned INTEGER NOT NULL DEFAULT 0,
                lifetime_spent INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS currency_transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                amount INTEGER NOT NULL,
                transaction_type TEXT NOT NULL,
                reason TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_currency_transactions_user ON currency_transactions(user_id);
            CREATE INDEX IF NOT EXISTS idx_currency_transactions_created ON currency_transactions(created_at);
            "#,
        ])?;

        // Register services
        ctx.provide_service("get_balance", |input| async move {
            let channel: String = serde_json::from_value(input.get("channel").cloned().unwrap_or_else(|| serde_json::json!("pianofire")))?;
            let username: String = serde_json::from_value(input.get("username").or(input.get("user_id")).cloned().unwrap_or_default())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let balance = database::get_balance(&conn, &channel, &username)?;
            Ok(serde_json::json!({ "balance": balance }))
        }).await;

        ctx.provide_service("add_currency", |input| async move {
            let channel: String = serde_json::from_value(input.get("channel").cloned().unwrap_or_else(|| serde_json::json!("pianofire")))?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let amount: i64 = serde_json::from_value(input["amount"].clone())?;
            let reason: Option<String> = serde_json::from_value(input["reason"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let new_balance = database::add_currency(&conn, &channel, &username, amount, reason.as_deref())?;
            Ok(serde_json::json!({ "success": true, "balance": new_balance }))
        }).await;

        ctx.provide_service("deduct_currency", |input| async move {
            let channel: String = serde_json::from_value(input.get("channel").cloned().unwrap_or_else(|| serde_json::json!("pianofire")))?;
            let username: String = serde_json::from_value(input.get("username").or(input.get("user_id")).cloned().unwrap_or_default())?;
            let amount: i64 = serde_json::from_value(input["amount"].clone())?;
            let reason: Option<String> = serde_json::from_value(input["reason"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let new_balance = database::deduct_currency(&conn, &channel, &username, amount, reason.as_deref())?;
            Ok(serde_json::json!({ "success": true, "balance": new_balance }))
        }).await;

        ctx.provide_service("transfer_currency", |input| async move {
            let channel: String = serde_json::from_value(input.get("channel").cloned().unwrap_or_else(|| serde_json::json!("pianofire")))?;
            let from_username: String = serde_json::from_value(input["from_username"].clone())?;
            let to_username: String = serde_json::from_value(input["to_username"].clone())?;
            let amount: i64 = serde_json::from_value(input["amount"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::transfer_currency(&conn, &channel, &from_username, &to_username, amount)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Currency] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Currency] Starting plugin...");

        // Subscribe to chat messages for commands
        let ctx_commands = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_commands.subscribe_to("twitch.chat_message").await;

            while let Ok(event) = events.recv().await {
                if let (Ok(channel), Ok(username), Ok(user_id), Ok(message)) = (
                    serde_json::from_value::<String>(event.payload["channel"].clone()),
                    serde_json::from_value::<String>(event.payload["username"].clone()),
                    serde_json::from_value::<String>(event.payload["user_id"].clone()),
                    serde_json::from_value::<String>(event.payload["message"].clone()),
                ) {
                    let parts: Vec<String> = message.split_whitespace().map(|s| s.to_string()).collect();
                    let ctx_cmd = ctx_commands.clone();

                    match parts.get(0).map(|s| s.as_str()) {
                        Some("!gamba") | Some("!gamble") => {
                            let args: Vec<String> = parts[1..].to_vec();
                            tokio::spawn(async move {
                                handle_gamba_command(&channel, &username, &user_id, &args, ctx_cmd).await;
                            });
                        }
                        Some("!coins") | Some("!balance") | Some("!bal") => {
                            let args: Vec<String> = parts[1..].to_vec();
                            tokio::spawn(async move {
                                handle_coins_command(&channel, &username, &user_id, &args, ctx_cmd).await;
                            });
                        }
                        Some("!givecoins") => {
                            // Check if user is mod/broadcaster
                            let is_mod = event.payload["is_mod"].as_bool().unwrap_or(false);
                            let is_broadcaster = event.payload["is_broadcaster"].as_bool().unwrap_or(false);

                            if is_mod || is_broadcaster {
                                let args: Vec<String> = parts[1..].to_vec();
                                tokio::spawn(async move {
                                    handle_give_coins_command(&channel, &args, ctx_cmd).await;
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        // Subscribe to events that award currency
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("twitch.follow").await;

            while let Ok(event) = events.recv().await {
                // Award currency for follows
                if let Ok(user_id) = serde_json::from_value::<String>(event.payload["user_id"].clone()) {
                    if let Ok(username) = serde_json::from_value::<String>(event.payload["username"].clone()) {
                        let _ = ctx_clone.call_service("currency", "add_currency", serde_json::json!({
                            "user_id": user_id,
                            "username": username,
                            "amount": 100,
                            "reason": "Follow reward"
                        })).await;

                        ctx_clone.emit("currency.earned", &CurrencyEarnedEvent {
                            user_id,
                            username,
                            amount: 100,
                            reason: "Follow reward".to_string(),
                        });
                    }
                }
            }
        });

        log::info!("[Currency] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Currency] Stopping plugin...");
        Ok(())
    }
}

async fn handle_gamba_command(channel: &str, username: &str, user_id: &str, args: &[String], ctx: Arc<PluginContext>) {
    if args.is_empty() {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": format!("@{} Usage: !gamba <amount> or !gamba all", username)
        })).await;
        return;
    }

    // Get user's current balance
    let balance_result = ctx.call_service("currency", "get_balance", serde_json::json!({
        "channel": channel,
        "username": username
    })).await;

    let current_balance = match balance_result {
        Ok(result) => result["balance"].as_i64().unwrap_or(0),
        _ => {
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": "❌ Failed to check your balance"
            })).await;
            return;
        }
    };

    if current_balance == 0 {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": format!("@{} You don't have any coins to gamble! Chat more to earn coins.", username)
        })).await;
        return;
    }

    // Parse bet amount
    let bet_amount = if args[0].to_lowercase() == "all" {
        current_balance
    } else {
        match args[0].parse::<i64>() {
            Ok(amount) if amount > 0 => amount,
            _ => {
                let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                    "channel": channel,
                    "message": format!("@{} Please enter a valid amount (or 'all')", username)
                })).await;
                return;
            }
        }
    };

    // Check if user has enough coins
    if bet_amount > current_balance {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": format!("@{} You don't have enough coins! Your balance: {} coins", username, current_balance)
        })).await;
        return;
    }

    // Minimum bet
    if bet_amount < 10 {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": format!("@{} Minimum bet is 10 coins!", username)
        })).await;
        return;
    }

    // Roll the dice (50/50 chance, but slightly favorable: 53%)
    let roll = fastrand::u8(1..=100);
    let won = roll >= 48; // 53% chance to win

    if won {
        // Win: add the bet amount (double their bet)
        let winnings = bet_amount;
        let add_result = ctx.call_service("currency", "add_currency", serde_json::json!({
            "channel": channel,
            "username": username,
            "amount": winnings,
            "reason": "Gamba winnings"
        })).await;

        if let Ok(result) = add_result {
            let new_balance = result["balance"].as_i64().unwrap_or(current_balance + winnings);
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": format!("🎰 @{} rolled {} and WON! +{} coins! New balance: {} coins", username, roll, winnings, new_balance)
            })).await;
        }
    } else {
        // Lose: remove the bet amount
        let deduct_result = ctx.call_service("currency", "deduct_currency", serde_json::json!({
            "channel": channel,
            "username": username,
            "amount": bet_amount,
            "reason": "Gamba loss"
        })).await;

        if let Ok(result) = deduct_result {
            let new_balance = result["balance"].as_i64().unwrap_or(current_balance - bet_amount);
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": format!("🎰 @{} rolled {} and lost. -{} coins. New balance: {} coins. Better luck next time!", username, roll, bet_amount, new_balance)
            })).await;
        }
    }
}

async fn handle_coins_command(channel: &str, username: &str, _user_id: &str, args: &[String], ctx: Arc<PluginContext>) {
    // Determine target user
    let target_username = if args.is_empty() {
        username.to_string()
    } else {
        args[0].trim_start_matches('@').to_string()
    };

    let balance_result = ctx.call_service("currency", "get_balance", serde_json::json!({
        "channel": channel,
        "username": target_username
    })).await;

    match balance_result {
        Ok(result) => {
            let coins = result["balance"].as_i64().unwrap_or(0);

            if target_username == username {
                let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                    "channel": channel,
                    "message": format!("💰 @{} Balance: {} coins", target_username, coins)
                })).await;
            } else {
                let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                    "channel": channel,
                    "message": format!("💰 {}'s Balance: {} coins", target_username, coins)
                })).await;
            }
        }
        _ => {
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": format!("❌ Failed to get balance for {}", target_username)
            })).await;
        }
    }
}

async fn handle_give_coins_command(channel: &str, args: &[String], ctx: Arc<PluginContext>) {
    if args.len() < 2 {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": "Usage: !givecoins <username> <amount>"
        })).await;
        return;
    }

    let target_username = args[0].trim_start_matches('@').to_string();

    let amount = match args[1].parse::<i64>() {
        Ok(amt) if amt > 0 && amt <= 100000 => amt,
        Ok(_) => {
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": "Amount must be between 1 and 100,000"
            })).await;
            return;
        }
        Err(_) => {
            let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
                "channel": channel,
                "message": "Invalid amount. Usage: !givecoins <username> <amount>"
            })).await;
            return;
        }
    };

    let add_result = ctx.call_service("currency", "add_currency", serde_json::json!({
        "channel": channel,
        "username": target_username,
        "amount": amount,
        "reason": "Admin gift"
    })).await;

    if add_result.is_ok() {
        // Get new balance
        let balance_result = ctx.call_service("currency", "get_balance", serde_json::json!({
            "channel": channel,
            "username": target_username
        })).await;

        let new_balance = balance_result.ok().and_then(|r| r["balance"].as_i64()).unwrap_or(0);

        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": format!("💰 Awarded {} coins to @{}! They now have {} coins total.", amount, target_username, new_balance)
        })).await;
    } else {
        let _ = ctx.call_service("twitch", "send_message", serde_json::json!({
            "channel": channel,
            "message": "❌ Failed to award coins"
        })).await;
    }
}

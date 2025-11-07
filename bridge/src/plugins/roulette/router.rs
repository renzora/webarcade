use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;
use std::sync::Arc;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    let ctx_start = Arc::new(ctx.clone());
    let ctx_bet = Arc::new(ctx.clone());
    let ctx_spin = Arc::new(ctx.clone());
    let ctx_result = Arc::new(ctx.clone());

    // POST /roulette/start - Start a new roulette game
    router.route(Method::POST, "/start", move |_path, _query, req| {
        let ctx = ctx_start.clone();
        Box::pin(async move {
            handle_start_game(req, ctx).await
        })
    });

    // GET /roulette/start?channel=xxx - Start a new roulette game via query param
    let ctx_start_get = Arc::new(ctx.clone());
    router.route(Method::GET, "/start", move |_path, query, _req| {
        let ctx = ctx_start_get.clone();
        Box::pin(async move {
            handle_start_game_query(query, ctx).await
        })
    });

    // POST /roulette/bet - Place a bet on the current game
    router.route(Method::POST, "/bet", move |_path, _query, req| {
        let ctx = ctx_bet.clone();
        Box::pin(async move {
            handle_place_bet(req, ctx).await
        })
    });

    // POST /roulette/spin - Spin the wheel and start spinning
    router.route(Method::POST, "/spin", move |_path, _query, req| {
        let ctx = ctx_spin.clone();
        Box::pin(async move {
            handle_spin_wheel(req, ctx).await
        })
    });

    // POST /roulette/result - Submit spin result from client
    router.route(Method::POST, "/result", move |_path, _query, req| {
        let ctx = ctx_result.clone();
        Box::pin(async move {
            handle_submit_result(req, ctx).await
        })
    });

    // GET /roulette/active/:channel - Get the active game for a channel
    router.route(Method::GET, "/active/:channel", |path, _query, _req| {
        Box::pin(async move {
            handle_get_active_game(path).await
        })
    });

    // POST /roulette/cancel - Cancel the current game
    router.route(Method::POST, "/cancel", |_path, _query, req| {
        Box::pin(async move {
            handle_cancel_game(req).await
        })
    });

    // GET /roulette/game?channel=xxx - Get active game by channel query param
    router.route(Method::GET, "/game", |_path, query, _req| {
        Box::pin(async move {
            handle_get_game_query(query).await
        })
    });

    // GET /roulette/history?channel=xxx - Get roulette game history
    router.route(Method::GET, "/history", |_path, query, _req| {
        Box::pin(async move {
            handle_get_history(query).await
        })
    });

    // OPTIONS for CORS preflight
    router.route(Method::OPTIONS, "/start", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/bet", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/spin", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/result", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/cancel", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });

    ctx.register_router("roulette", router).await;
    Ok(())
}

async fn handle_start_game(req: Request<Incoming>, ctx: Arc<PluginContext>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::start_game(&conn, channel) {
                        Ok(game_id) => {
                            // Emit event that game started
                            ctx.emit("roulette.game_started", &serde_json::json!({
                                "channel": channel,
                                "game_id": game_id,
                                "timer_seconds": 30
                            }));

                            // Start 30-second auto-spin timer
                            let ctx_timer = ctx.clone();
                            let channel_clone = channel.to_string();
                            tokio::spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                                // Check if game is still in betting state
                                let conn_path = crate::core::database::get_database_path();
                                if let Ok(conn) = rusqlite::Connection::open(&conn_path) {
                                    if let Ok(Some(game)) = super::database::get_active_game(&conn, &channel_clone) {
                                        if game.status == "betting" {
                                            // Auto-spin
                                            let _ = super::database::start_spin(&conn, game.id);
                                            ctx_timer.emit("roulette.spin_started", &serde_json::json!({
                                                "channel": channel_clone,
                                                "game_id": game.id
                                            }));
                                        }
                                    }
                                }
                            });

                            json_response(&serde_json::json!({
                                "success": true,
                                "game_id": game_id
                            }))
                        },
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_start_game_query(query: String, ctx: Arc<PluginContext>) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(ch) => ch,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::start_game(&conn, &channel) {
                Ok(game_id) => {
                    // Emit event that game started
                    ctx.emit("roulette.game_started", &serde_json::json!({
                        "channel": channel.clone(),
                        "game_id": game_id,
                        "timer_seconds": 30
                    }));

                    // Start 30-second auto-spin timer
                    let ctx_timer = ctx.clone();
                    let channel_clone = channel.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                        // Check if game is still in betting state
                        let conn_path = crate::core::database::get_database_path();
                        if let Ok(conn) = rusqlite::Connection::open(&conn_path) {
                            if let Ok(Some(game)) = super::database::get_active_game(&conn, &channel_clone) {
                                if game.status == "betting" {
                                    // Auto-spin
                                    let _ = super::database::start_spin(&conn, game.id);
                                    ctx_timer.emit("roulette.spin_started", &serde_json::json!({
                                        "channel": channel_clone,
                                        "game_id": game.id
                                    }));
                                }
                            }
                        }
                    });

                    json_response(&serde_json::json!({
                        "success": true,
                        "game_id": game_id
                    }))
                },
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_place_bet(req: Request<Incoming>, ctx: Arc<PluginContext>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let amount = match body.get("amount").and_then(|v| v.as_i64()) {
                Some(amt) => amt,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing amount"),
            };
            let bet_type = match body.get("bet_type").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing bet_type"),
            };
            let bet_value = match body.get("bet_value").and_then(|v| v.as_str()) {
                Some(v) => v,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing bet_value"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::place_bet(&conn, channel, user_id, username, amount, bet_type, bet_value) {
                        Ok(_) => {
                            // Emit bet placed event
                            ctx.emit("roulette.bet_placed", &serde_json::json!({
                                "channel": channel,
                                "user_id": user_id,
                                "username": username,
                                "amount": amount,
                                "bet_type": bet_type,
                                "bet_value": bet_value
                            }));

                            json_response(&serde_json::json!({ "success": true }))
                        },
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_spin_wheel(req: Request<Incoming>, ctx: Arc<PluginContext>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get the active game
                    match super::database::get_active_game(&conn, channel) {
                        Ok(Some(game)) => {
                            // Mark game as spinning
                            match super::database::start_spin(&conn, game.id) {
                                Ok(_) => {
                                    // Emit spin started event
                                    ctx.emit("roulette.spin_started", &serde_json::json!({
                                        "channel": channel,
                                        "game_id": game.id
                                    }));

                                    json_response(&serde_json::json!({
                                        "success": true,
                                        "game_id": game.id
                                    }))
                                },
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                            }
                        },
                        Ok(None) => error_response(StatusCode::NOT_FOUND, "No active game found"),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_submit_result(req: Request<Incoming>, ctx: Arc<PluginContext>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let game_id = match body.get("game_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing game_id"),
            };
            let winning_number = match body.get("winning_number").and_then(|v| v.as_i64()) {
                Some(num) => num,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing winning_number"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Submit the result
                    match super::database::set_spin_result(&conn, game_id, winning_number) {
                        Ok(_) => {
                            // Get bets to calculate winners and pay out
                            let bets = super::database::get_game_bets(&conn, game_id).unwrap_or_default();

                            // Calculate totals and pay winners
                            let mut total_wagered: i64 = 0;
                            let mut total_payout: i64 = 0;

                            for bet in &bets {
                                total_wagered += bet.amount;

                                if let Some(payout) = bet.payout {
                                    if payout > 0 {
                                        // Pay the winner via currency plugin
                                        let ctx_clone = ctx.clone();
                                        let username = bet.username.clone();
                                        let channel_name = channel.to_string();
                                        tokio::spawn(async move {
                                            let _ = ctx_clone.call_service("currency", "add_currency", serde_json::json!({
                                                "channel": channel_name,
                                                "username": username,
                                                "amount": payout,
                                                "reason": "Roulette winnings"
                                            })).await;
                                        });
                                        total_payout += payout;
                                    }
                                }
                            }

                            // Get winning color
                            let winning_color = if winning_number == 0 {
                                "green"
                            } else {
                                let red_numbers = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
                                if red_numbers.contains(&(winning_number as i32)) {
                                    "red"
                                } else {
                                    "black"
                                }
                            };

                            // Emit result event
                            ctx.emit("roulette.result", &serde_json::json!({
                                "channel": channel,
                                "game_id": game_id,
                                "winning_number": winning_number,
                                "winning_color": winning_color,
                                "total_wagered": total_wagered,
                                "total_payout": total_payout
                            }));

                            json_response(&serde_json::json!({
                                "success": true,
                                "winning_number": winning_number,
                                "winning_color": winning_color,
                                "total_wagered": total_wagered,
                                "total_payout": total_payout
                            }))
                        },
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_active_game(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = extract_path_param(&path, "/active/");
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_active_game(&conn, &channel) {
                Ok(Some(game)) => json_response(&game),
                Ok(None) => error_response(StatusCode::NOT_FOUND, "No active game found"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_cancel_game(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::cancel_game(&conn, channel) {
                        Ok(_) => json_response(&serde_json::json!({ "success": true })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_game_query(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(ch) => ch,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_active_game(&conn, &channel) {
                Ok(Some(game)) => {
                    // Get bets for this game
                    let bets = super::database::get_game_bets(&conn, game.id).unwrap_or_default();

                    json_response(&serde_json::json!({
                        "game": game,
                        "bets": bets
                    }))
                },
                Ok(None) => json_response(&serde_json::json!({
                    "game": null,
                    "bets": []
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_history(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(ch) => ch,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, channel, winning_number, created_at, completed_at
                 FROM roulette_games
                 WHERE channel = ?1 AND status = 'ended'
                 ORDER BY completed_at DESC LIMIT 50"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(rusqlite::params![channel], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "channel": row.get::<_, String>(1)?,
                    "winning_number": row.get::<_, Option<i64>>(2)?,
                    "created_at": row.get::<_, i64>(3)?,
                    "completed_at": row.get::<_, Option<i64>>(4)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let games: Result<Vec<_>, _> = mapped.collect();

            match games {
                Ok(games) => json_response(&games),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

// Helper functions
fn parse_query_param(query: &str, key: &str) -> Option<String> {
    query.split('&')
        .find_map(|pair| {
            let mut parts = pair.split('=');
            if parts.next()? == key {
                Some(urlencoding::decode(parts.next()?).ok()?.into_owned())
            } else {
                None
            }
        })
}
fn extract_path_param(path: &str, prefix: &str) -> String {
    path.strip_prefix(prefix)
        .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
        .unwrap_or_default()
}

async fn read_json_body(req: Request<Incoming>) -> std::result::Result<serde_json::Value, String> {
    use http_body_util::BodyExt;
    let whole_body = req.collect().await
        .map_err(|e| format!("Failed to read body: {}", e))?
        .to_bytes();

    serde_json::from_slice(&whole_body)
        .map_err(|e| format!("Invalid JSON: {}", e))
}

fn json_response<T: serde::Serialize>(data: &T) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

fn error_response(status: StatusCode, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::json!({"error": message}).to_string();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

fn full_body(s: &str) -> BoxBody<Bytes, Infallible> {
    use http_body_util::combinators::BoxBody;
    use http_body_util::BodyExt;
    BoxBody::new(Full::new(Bytes::from(s.to_string())).map_err(|err: Infallible| match err {}))
}

fn cors_preflight_response() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(full_body(""))
        .unwrap()
}

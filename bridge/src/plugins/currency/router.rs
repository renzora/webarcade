use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /currency/balance/:channel/:username - Get user balance
    router.route(Method::GET, "/balance/:channel/:username", |path, _query, _req| {
        Box::pin(async move {
            handle_get_balance(path).await
        })
    });

    // POST /currency/add - Add currency to user
    router.route(Method::POST, "/add", |_path, _query, req| {
        Box::pin(async move {
            handle_add_currency(req).await
        })
    });

    // POST /currency/deduct - Deduct currency from user
    router.route(Method::POST, "/deduct", |_path, _query, req| {
        Box::pin(async move {
            handle_deduct_currency(req).await
        })
    });

    // POST /currency/transfer - Transfer currency between users
    router.route(Method::POST, "/transfer", |_path, _query, req| {
        Box::pin(async move {
            handle_transfer_currency(req).await
        })
    });

    // GET /currency/leaderboard - Get top users by balance
    router.route(Method::GET, "/leaderboard", |_path, query, _req| {
        Box::pin(async move {
            handle_get_leaderboard(query).await
        })
    });

    // GET /currency/transactions/:user_id - Get user transaction history
    router.route(Method::GET, "/transactions/:user_id", |path, query, _req| {
        Box::pin(async move {
            handle_get_transactions(path, query).await
        })
    });

    ctx.register_router("currency", router).await;
    Ok(())
}

async fn handle_get_balance(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 4 {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel or username");
    }
    let channel = urlencoding::decode(parts[2]).unwrap_or_default();
    let username = urlencoding::decode(parts[3]).unwrap_or_default();

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_balance(&conn, &channel, &username) {
                Ok(balance) => json_response(&serde_json::json!({ "balance": balance })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_currency(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => "pianofire",
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let amount = match body.get("amount").and_then(|v| v.as_i64()) {
                Some(amt) => amt,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing amount"),
            };
            let reason = body.get("reason").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::add_currency(&conn, channel, username, amount, reason) {
                        Ok(new_balance) => json_response(&serde_json::json!({ "success": true, "balance": new_balance })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_deduct_currency(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => "pianofire",
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let amount = match body.get("amount").and_then(|v| v.as_i64()) {
                Some(amt) => amt,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing amount"),
            };
            let reason = body.get("reason").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::deduct_currency(&conn, channel, username, amount, reason) {
                        Ok(new_balance) => json_response(&serde_json::json!({ "success": true, "balance": new_balance })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_transfer_currency(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => "pianofire",
            };
            let from_username = match body.get("from_username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing from_username"),
            };
            let to_username = match body.get("to_username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing to_username"),
            };
            let amount = match body.get("amount").and_then(|v| v.as_i64()) {
                Some(amt) => amt,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing amount"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::transfer_currency(&conn, channel, from_username, to_username, amount) {
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

async fn handle_get_leaderboard(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(10);
    let channel = parse_query_param(&query, "channel")
        .unwrap_or_else(|| "pianofire".to_string());

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT channel, username, coins FROM users WHERE channel = ?1 ORDER BY coins DESC LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([&channel, &limit.to_string()], |row| {
                Ok(serde_json::json!({
                    "channel": row.get::<_, String>(0)?,
                    "username": row.get::<_, String>(1)?,
                    "balance": row.get::<_, i64>(2)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let users: Result<Vec<serde_json::Value>, _> = mapped.collect();

            match users {
                Ok(users) => json_response(&serde_json::json!({ "leaderboard": users })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_transactions(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/transactions/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, amount, transaction_type, reason, created_at
                 FROM currency_transactions WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([&user_id, &limit.to_string()], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "amount": row.get::<_, i64>(1)?,
                    "transaction_type": row.get::<_, String>(2)?,
                    "reason": row.get::<_, Option<String>>(3)?,
                    "created_at": row.get::<_, i64>(4)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let transactions: Result<Vec<serde_json::Value>, _> = mapped.collect();

            match transactions {
                Ok(txs) => json_response(&serde_json::json!({ "transactions": txs })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

// Helper functions
fn extract_path_param(path: &str, prefix: &str) -> String {
    path.strip_prefix(prefix)
        .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
        .unwrap_or_default()
}

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

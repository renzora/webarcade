use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /auction/active - Get all active auctions
    router.route(Method::GET, "/active", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_active_auctions().await
        })
    });

    // GET /auction/:auction_id - Get a specific auction
    router.route(Method::GET, "/:auction_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_auction(path).await
        })
    });

    // POST /auction/create - Create a new auction
    router.route(Method::POST, "/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_auction(req).await
        })
    });

    // POST /auction/bid - Place a bid on an auction
    router.route(Method::POST, "/bid", |_path, _query, req| {
        Box::pin(async move {
            handle_place_bid(req).await
        })
    });

    // GET /auction/:auction_id/bids - Get all bids for an auction
    router.route(Method::GET, "/:auction_id/bids", |path, query, _req| {
        Box::pin(async move {
            handle_get_auction_bids(path, query).await
        })
    });

    ctx.register_router("auction", router).await;
    Ok(())
}

async fn handle_get_active_auctions() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_active_auctions(&conn) {
                Ok(auctions) => json_response(&serde_json::json!({ "auctions": auctions })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_auction(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let auction_id_str = extract_path_param(&path, "/");
    let auction_id = match auction_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid auction_id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, item_id, item_name, item_rarity, starting_bid, current_bid,
                        current_bidder, created_by, status, created_at, ends_at, ended_at
                 FROM auctions WHERE id = ?1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let result = stmt.query_row([auction_id], |row| {
                Ok(super::database::Auction {
                    id: row.get(0)?,
                    item_id: row.get(1)?,
                    item_name: row.get(2)?,
                    item_rarity: row.get(3)?,
                    starting_bid: row.get(4)?,
                    current_bid: row.get(5)?,
                    current_bidder: row.get(6)?,
                    created_by: row.get(7)?,
                    status: row.get(8)?,
                    created_at: row.get(9)?,
                    ends_at: row.get(10)?,
                    ended_at: row.get(11)?,
                })
            });

            match result {
                Ok(auction) => json_response(&serde_json::json!({ "auction": auction })),
                Err(rusqlite::Error::QueryReturnedNoRows) => error_response(StatusCode::NOT_FOUND, "Auction not found"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_create_auction(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let item_id = match body.get("item_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing item_id"),
            };
            let item_name = match body.get("item_name").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing item_name"),
            };
            let item_rarity = match body.get("item_rarity").and_then(|v| v.as_str()) {
                Some(rarity) => rarity,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing item_rarity"),
            };
            let starting_bid = match body.get("starting_bid").and_then(|v| v.as_i64()) {
                Some(bid) => bid,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing starting_bid"),
            };
            let created_by = match body.get("created_by").and_then(|v| v.as_str()) {
                Some(creator) => creator,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing created_by"),
            };
            let duration_seconds = match body.get("duration_seconds").and_then(|v| v.as_i64()) {
                Some(duration) => duration,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing duration_seconds"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::create_auction(
                        &conn,
                        item_id,
                        item_name,
                        item_rarity,
                        starting_bid,
                        created_by,
                        duration_seconds,
                    ) {
                        Ok(auction_id) => json_response(&serde_json::json!({ "auction_id": auction_id })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_place_bid(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let auction_id = match body.get("auction_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing auction_id"),
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

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::place_bid(&conn, auction_id, user_id, username, amount) {
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

async fn handle_get_auction_bids(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let auction_id_str = extract_path_param(&path, "/");
    let auction_id_str = auction_id_str.trim_end_matches("/bids");
    let auction_id = match auction_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid auction_id"),
    };

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, auction_id, user_id, username, amount, created_at
                 FROM auction_bids WHERE auction_id = ?1 ORDER BY created_at DESC LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let bids: Result<Vec<serde_json::Value>, _> = stmt.query_map([auction_id, limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "auction_id": row.get::<_, i64>(1)?,
                    "user_id": row.get::<_, String>(2)?,
                    "username": row.get::<_, String>(3)?,
                    "amount": row.get::<_, i64>(4)?,
                    "created_at": row.get::<_, i64>(5)?,
                }))
            }).and_then(|rows| rows.collect());

            match bids {
                Ok(bids) => json_response(&serde_json::json!({ "bids": bids })),
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

use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /packs - Get all available packs (root endpoint)
    router.route(Method::GET, "/", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_packs().await
        })
    });

    // GET /packs/list - Get all available packs
    router.route(Method::GET, "/list", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_packs().await
        })
    });

    // GET /packs/items - Get all pack items
    router.route(Method::GET, "/items", |_path, query, _req| {
        Box::pin(async move {
            handle_get_pack_items(query).await
        })
    });

    // POST /packs/create - Create a new pack definition
    router.route(Method::POST, "/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_pack(req).await
        })
    });

    // POST /packs/add-item - Add an item to a pack
    router.route(Method::POST, "/add-item", |_path, _query, req| {
        Box::pin(async move {
            handle_add_pack_item(req).await
        })
    });

    // POST /packs/open - Open a pack for a user
    router.route(Method::POST, "/open", |_path, _query, req| {
        Box::pin(async move {
            handle_open_pack(req).await
        })
    });

    // GET /packs/inventory/:user_id - Get user's inventory
    router.route(Method::GET, "/inventory/:user_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_inventory(path).await
        })
    });

    // GET /packs/history/:user_id - Get user's pack opening history
    router.route(Method::GET, "/history/:user_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_history(path).await
        })
    });

    // OPTIONS for CORS preflight
    router.route(Method::OPTIONS, "/create", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/add-item", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/open", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });

    ctx.register_router("packs", router).await;
    Ok(())
}

async fn handle_get_packs() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, name, description, cost, image_url FROM pack_definitions WHERE enabled = 1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "cost": row.get::<_, i64>(3)?,
                    "image_url": row.get::<_, Option<String>>(4)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let packs: Result<Vec<_>, _> = mapped.collect();

            match packs {
                Ok(packs) => json_response(&serde_json::json!({ "packs": packs })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_pack_items(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let pack_id = parse_query_param(&query, "pack_id").and_then(|s| s.parse::<i64>().ok());

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let query_str = if let Some(pack_id) = pack_id {
                format!("SELECT id, pack_id, item_name, item_description, rarity, weight FROM pack_items WHERE pack_id = {}", pack_id)
            } else {
                "SELECT id, pack_id, item_name, item_description, rarity, weight FROM pack_items".to_string()
            };

            let mut stmt = match conn.prepare(&query_str) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "pack_id": row.get::<_, i64>(1)?,
                    "item_name": row.get::<_, String>(2)?,
                    "item_description": row.get::<_, Option<String>>(3)?,
                    "rarity": row.get::<_, String>(4)?,
                    "weight": row.get::<_, i64>(5)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let items: Result<Vec<_>, _> = mapped.collect();

            match items {
                Ok(items) => json_response(&serde_json::json!({ "items": items })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_create_pack(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let name = match body.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing name"),
            };
            let description = match body.get("description").and_then(|v| v.as_str()) {
                Some(d) => d,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing description"),
            };
            let cost = match body.get("cost").and_then(|v| v.as_i64()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing cost"),
            };
            let image_url = body.get("image_url").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO pack_definitions (name, description, cost, enabled, image_url, created_at, updated_at)
                         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?5)",
                        rusqlite::params![name, description, cost, image_url, now],
                    ) {
                        Ok(_) => {
                            let pack_id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": pack_id, "success": true }))
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_add_pack_item(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let pack_id = match body.get("pack_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing pack_id"),
            };
            let item_name = match body.get("item_name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing item_name"),
            };
            let item_description = body.get("item_description").and_then(|v| v.as_str());
            let rarity = match body.get("rarity").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing rarity"),
            };
            let weight = body.get("weight").and_then(|v| v.as_i64()).unwrap_or(1);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO pack_items (pack_id, item_name, item_description, rarity, weight, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![pack_id, item_name, item_description, rarity, weight, now],
                    ) {
                        Ok(_) => {
                            let item_id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": item_id, "success": true }))
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_open_pack(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let pack_id = match body.get("pack_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing pack_id"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get pack info
                    let pack_info = conn.query_row(
                        "SELECT name, description, cost FROM pack_definitions WHERE id = ?1 AND enabled = 1",
                        rusqlite::params![pack_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, i64>(2)?,
                            ))
                        },
                    );

                    let (pack_name, _pack_desc, cost) = match pack_info {
                        Ok(info) => info,
                        Err(e) => return error_response(StatusCode::NOT_FOUND, &format!("Pack not found: {}", e)),
                    };

                    // Get all items in pack with weights
                    let mut stmt = match conn.prepare(
                        "SELECT item_name, item_description, rarity, weight FROM pack_items WHERE pack_id = ?1"
                    ) {
                        Ok(stmt) => stmt,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    let mapped = match stmt.query_map(
                        rusqlite::params![pack_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                    ) {
                        Ok(m) => m,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    let items: Result<Vec<_>, _> = mapped.collect();

                    let items = match items {
                        Ok(items) if !items.is_empty() => items,
                        Ok(_) => return error_response(StatusCode::BAD_REQUEST, "Pack has no items"),
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    // Select random items based on weights (3 items per pack)
                    let mut obtained_items = Vec::new();
                    let items_to_draw = 3.min(items.len());

                    for _ in 0..items_to_draw {
                        let selected = select_weighted_item(&items);
                        obtained_items.push(selected.clone());
                    }

                    // Add items to user inventory
                    let now = current_timestamp();
                    for (item_name, item_desc, rarity, _) in &obtained_items {
                        if let Err(e) = conn.execute(
                            "INSERT INTO user_inventory (user_id, username, item_name, item_description, rarity, pack_name, obtained_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![user_id, username, item_name, item_desc, rarity, pack_name, now],
                        ) {
                            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                        }
                    }

                    // Record pack opening
                    let items_json = match serde_json::to_string(&obtained_items) {
                        Ok(json) => json,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    if let Err(e) = conn.execute(
                        "INSERT INTO pack_openings (user_id, username, pack_id, pack_name, items_obtained, cost, opened_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![user_id, username, pack_id, pack_name, items_json, cost, now],
                    ) {
                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                    }

                    let response = serde_json::json!({
                        "success": true,
                        "items": obtained_items.iter().map(|(name, desc, rarity, _)| {
                            serde_json::json!({
                                "name": name,
                                "description": desc,
                                "rarity": rarity
                            })
                        }).collect::<Vec<_>>(),
                        "cost": cost
                    });

                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_inventory(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/inventory/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, item_name, item_description, rarity, pack_name, obtained_at
                 FROM user_inventory WHERE user_id = ?1 ORDER BY obtained_at DESC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(rusqlite::params![user_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "item_name": row.get::<_, String>(1)?,
                    "item_description": row.get::<_, Option<String>>(2)?,
                    "rarity": row.get::<_, String>(3)?,
                    "pack_name": row.get::<_, String>(4)?,
                    "obtained_at": row.get::<_, i64>(5)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let items: Result<Vec<_>, _> = mapped.collect();

            match items {
                Ok(items) => {
                    let total = items.len();
                    json_response(&serde_json::json!({ "items": items, "total": total }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_history(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/history/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, pack_name, items_obtained, cost, opened_at
                 FROM pack_openings WHERE user_id = ?1 ORDER BY opened_at DESC LIMIT 50"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(rusqlite::params![user_id], |row| {
                let items_json: String = row.get(2)?;
                let items: Vec<serde_json::Value> = serde_json::from_str(&items_json).unwrap_or_default();

                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "pack_name": row.get::<_, String>(1)?,
                    "items": items,
                    "cost": row.get::<_, i64>(3)?,
                    "opened_at": row.get::<_, i64>(4)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let openings: Result<Vec<_>, _> = mapped.collect();

            match openings {
                Ok(openings) => json_response(&serde_json::json!({ "openings": openings })),
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

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn select_weighted_item(items: &[(String, Option<String>, String, i64)]) -> (String, Option<String>, String, i64) {
    use rand::Rng;

    let total_weight: i64 = items.iter().map(|(_, _, _, w)| w).sum();
    let mut rng = rand::thread_rng();
    let mut roll = rng.gen_range(0..total_weight);

    for item in items {
        if roll < item.3 {
            return item.clone();
        }
        roll -= item.3;
    }

    items[0].clone()
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

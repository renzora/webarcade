use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // POST /hue/bridge/register - Register a Hue bridge
    router.route(Method::POST, "/bridge/register", |_path, _query, req| {
        Box::pin(async move {
            handle_register_bridge(req).await
        })
    });

    // POST /hue/light/register - Register a light
    router.route(Method::POST, "/light/register", |_path, _query, req| {
        Box::pin(async move {
            handle_register_light(req).await
        })
    });

    // GET /hue/lights - Get all lights (optionally filtered by bridge_id)
    router.route(Method::GET, "/lights", |_path, query, _req| {
        Box::pin(async move {
            handle_get_lights(query).await
        })
    });

    // POST /hue/light/state - Set light state (on/off, brightness, color)
    router.route(Method::POST, "/light/state", |_path, _query, req| {
        Box::pin(async move {
            handle_set_light_state(req).await
        })
    });

    // POST /hue/scene/create - Create a new scene
    router.route(Method::POST, "/scene/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_scene(req).await
        })
    });

    // GET /hue/scenes - Get all scenes
    router.route(Method::GET, "/scenes", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_scenes().await
        })
    });

    // POST /hue/scene/activate - Activate a scene
    router.route(Method::POST, "/scene/activate", |_path, _query, req| {
        Box::pin(async move {
            handle_activate_scene(req).await
        })
    });

    // POST /hue/automation/create - Create automation
    router.route(Method::POST, "/automation/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_automation(req).await
        })
    });

    // GET /hue/config - Get Hue bridge configuration
    router.route(Method::GET, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_config().await
        })
    });

    // GET /hue/animated-scenes - Get all animated scenes
    router.route(Method::GET, "/animated-scenes", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_animated_scenes().await
        })
    });

    // POST /hue/animated-scene/create - Create animated scene
    router.route(Method::POST, "/animated-scene/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_animated_scene(req).await
        })
    });

    // OPTIONS for CORS preflight
    router.route(Method::OPTIONS, "/bridge/register", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/light/register", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/light/state", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/scene/create", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/scene/activate", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/automation/create", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/animated-scene/create", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });

    ctx.register_router("hue", router).await;
    Ok(())
}

async fn handle_register_bridge(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let bridge_id = match body.get("bridge_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing bridge_id"),
            };
            let ip_address = match body.get("ip_address").and_then(|v| v.as_str()) {
                Some(ip) => ip,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing ip_address"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(user) => user,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let name = match body.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing name"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR REPLACE INTO hue_bridges (bridge_id, ip_address, username, name, enabled, created_at)
                         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
                        rusqlite::params![bridge_id, ip_address, username, name, now],
                    ) {
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

async fn handle_register_light(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let bridge_id = match body.get("bridge_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing bridge_id"),
            };
            let light_id = match body.get("light_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing light_id"),
            };
            let light_name = match body.get("light_name").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing light_name"),
            };
            let light_type = match body.get("light_type").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing light_type"),
            };
            let capabilities = body.get("capabilities").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR REPLACE INTO hue_lights (bridge_id, light_id, light_name, light_type, capabilities, enabled, last_seen, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
                        rusqlite::params![bridge_id, light_id, light_name, light_type, capabilities, now],
                    ) {
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

async fn handle_get_lights(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let bridge_id = parse_query_param(&query, "bridge_id");

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut sql = "SELECT id, bridge_id, light_id, light_name, light_type, enabled FROM hue_lights WHERE 1=1".to_string();
            let mut params: Vec<String> = Vec::new();

            if let Some(ref bid) = bridge_id {
                sql.push_str(" AND bridge_id = ?");
                params.push(bid.clone());
            }

            sql.push_str(" AND enabled = 1 ORDER BY light_name");

            let mut stmt = match conn.prepare(&sql) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

            let mapped = match stmt.query_map(param_refs.as_slice(), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "bridge_id": row.get::<_, String>(1)?,
                    "light_id": row.get::<_, String>(2)?,
                    "light_name": row.get::<_, String>(3)?,
                    "light_type": row.get::<_, String>(4)?,
                    "enabled": row.get::<_, i64>(5)? != 0,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let lights: Result<Vec<_>, _> = mapped.collect();

            match lights {
                Ok(lights) => json_response(&serde_json::json!({ "lights": lights })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_set_light_state(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let light_id = match body.get("light_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing light_id"),
            };
            let on = body.get("on").and_then(|v| v.as_bool());
            let brightness = body.get("brightness").and_then(|v| v.as_i64());
            let color = body.get("color").and_then(|v| v.as_str());
            let triggered_by = body.get("triggered_by").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let action_data = serde_json::json!({
                        "on": on,
                        "brightness": brightness,
                        "color": color
                    });

                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO hue_history (action_type, light_id, action_data, triggered_by, timestamp)
                         VALUES ('set_state', ?1, ?2, ?3, ?4)",
                        rusqlite::params![light_id, action_data.to_string(), triggered_by, now],
                    ) {
                        Ok(_) => json_response(&serde_json::json!({
                            "success": true,
                            "light_id": light_id,
                            "state": action_data
                        })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_create_scene(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let scene_name = match body.get("scene_name").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing scene_name"),
            };
            let scene_data = match body.get("scene_data").and_then(|v| v.as_str()) {
                Some(data) => data,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing scene_data"),
            };
            let description = body.get("description").and_then(|v| v.as_str());
            let created_by = body.get("created_by").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO hue_scenes (scene_name, scene_data, description, created_by, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![scene_name, scene_data, description, created_by, now],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": id, "success": true }))
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

async fn handle_get_scenes() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, name, red, green, blue, created_at FROM hue_scenes ORDER BY created_at DESC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "red": row.get::<_, i64>(2)?,
                    "green": row.get::<_, i64>(3)?,
                    "blue": row.get::<_, i64>(4)?,
                    "created_at": row.get::<_, i64>(5)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let scenes: Result<Vec<_>, _> = mapped.collect();

            match scenes {
                Ok(scenes) => json_response(&serde_json::json!({ "scenes": scenes })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_activate_scene(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let scene_id = match body.get("scene_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing scene_id"),
            };
            let triggered_by = body.get("triggered_by").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get scene data
                    let scene_result: rusqlite::Result<(String, String)> = conn.query_row(
                        "SELECT scene_name, scene_data FROM hue_scenes WHERE id = ?1",
                        rusqlite::params![scene_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    );

                    match scene_result {
                        Ok((scene_name, scene_data)) => {
                            // Log to history
                            let now = current_timestamp();
                            match conn.execute(
                                "INSERT INTO hue_history (action_type, scene_id, action_data, triggered_by, timestamp)
                                 VALUES ('activate_scene', ?1, ?2, ?3, ?4)",
                                rusqlite::params![scene_id, scene_data, triggered_by, now],
                            ) {
                                Ok(_) => json_response(&serde_json::json!({
                                    "success": true,
                                    "scene_name": scene_name
                                })),
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                            }
                        }
                        Err(e) => error_response(StatusCode::NOT_FOUND, &format!("Scene not found: {}", e)),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_create_automation(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let automation_name = match body.get("automation_name").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing automation_name"),
            };
            let trigger_event = match body.get("trigger_event").and_then(|v| v.as_str()) {
                Some(event) => event,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing trigger_event"),
            };
            let scene_id = body.get("scene_id").and_then(|v| v.as_i64());
            let light_ids = body.get("light_ids").and_then(|v| v.as_str());
            let action_data = match body.get("action_data").and_then(|v| v.as_str()) {
                Some(data) => data,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing action_data"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO hue_automations (automation_name, trigger_event, scene_id, light_ids, action_data, enabled, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                        rusqlite::params![automation_name, trigger_event, scene_id, light_ids, action_data, now],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": id, "success": true }))
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

async fn handle_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT bridge_id, ip_address, username, name, enabled FROM hue_bridges WHERE enabled = 1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "bridge_id": row.get::<_, String>(0)?,
                    "ip_address": row.get::<_, String>(1)?,
                    "username": row.get::<_, String>(2)?,
                    "name": row.get::<_, String>(3)?,
                    "enabled": row.get::<_, i64>(4)? != 0,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let bridges: Result<Vec<_>, _> = mapped.collect();

            match bridges {
                Ok(bridges) => json_response(&serde_json::json!({ "bridges": bridges })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_animated_scenes() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, name, tag, created_at FROM hue_animated_scenes ORDER BY created_at DESC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "tag": row.get::<_, String>(2)?,
                    "created_at": row.get::<_, i64>(3)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let scenes: Result<Vec<_>, _> = mapped.collect();

            match scenes {
                Ok(scenes) => json_response(&scenes),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_create_animated_scene(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let scene_name = match body.get("scene_name").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing scene_name"),
            };
            let frames_data = match body.get("frames_data").and_then(|v| v.as_str()) {
                Some(data) => data,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing frames_data"),
            };
            let duration_ms = match body.get("duration_ms").and_then(|v| v.as_i64()) {
                Some(dur) => dur,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing duration_ms"),
            };
            let description = body.get("description").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO hue_animated_scenes (scene_name, description, frames_data, duration_ms, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![scene_name, description, frames_data, duration_ms, now],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": id, "success": true }))
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

fn cors_preflight_response() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(full_body(""))
        .unwrap()
}

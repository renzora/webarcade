use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use anyhow::Result;
use hyper::{Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::combinators::BoxBody;
use std::convert::Infallible;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct BridgeConfig {
    name: String,
    ip_address: String,
    username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LightState {
    on: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    bri: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hue: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sat: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ct: Option<u16>,
}

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // Bridge management
    route!(router, GET "/bridges" => handle_get_bridges);
    route!(router, POST "/bridges" => handle_add_bridge);
    route!(router, DELETE "/bridges/:bridge_id", path => handle_delete_bridge);
    route!(router, POST "/bridges/:bridge_id/pair", path => handle_pair_bridge);
    route!(router, GET "/bridges/:bridge_id/discover", path => handle_discover_lights);

    // Light control
    route!(router, GET "/lights" => handle_get_lights);
    route!(router, GET "/lights/:light_id", path => handle_get_light);
    route!(router, PUT "/lights/:light_id/state", path => handle_set_light_state);

    // Group/Room control
    route!(router, GET "/groups" => handle_get_groups);
    route!(router, PUT "/groups/:group_id/state", path => handle_set_group_state);

    // CORS preflight handlers
    route!(router, OPTIONS "/bridges" => cors_preflight);
    route!(router, OPTIONS "/bridges/:bridge_id" => cors_preflight);
    route!(router, OPTIONS "/bridges/:bridge_id/pair" => cors_preflight);
    route!(router, OPTIONS "/bridges/:bridge_id/discover" => cors_preflight);
    route!(router, OPTIONS "/lights" => cors_preflight);
    route!(router, OPTIONS "/lights/:light_id" => cors_preflight);
    route!(router, OPTIONS "/lights/:light_id/state" => cors_preflight);
    route!(router, OPTIONS "/groups" => cors_preflight);
    route!(router, OPTIONS "/groups/:group_id/state" => cors_preflight);

    ctx.register_router("philips-hue", router).await;
    Ok(())
}

async fn handle_get_bridges() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare("SELECT id, name, ip_address, username, created_at, last_connected FROM hue_bridges ORDER BY created_at DESC") {
                Ok(s) => s,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let bridges: Vec<serde_json::Value> = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "ip_address": row.get::<_, String>(2)?,
                    "username": row.get::<_, Option<String>>(3)?,
                    "created_at": row.get::<_, i64>(4)?,
                    "last_connected": row.get::<_, Option<i64>>(5)?,
                }))
            }).and_then(|mapped| mapped.collect()) {
                Ok(b) => b,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            json_response(&serde_json::json!({ "bridges": bridges }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_bridge(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let config: BridgeConfig = match serde_json::from_value(body) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            match conn.execute(
                "INSERT INTO hue_bridges (name, ip_address, username, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![config.name, config.ip_address, config.username, now],
            ) {
                Ok(_) => {
                    let bridge_id = conn.last_insert_rowid();
                    json_response(&serde_json::json!({
                        "success": true,
                        "bridge_id": bridge_id,
                        "message": "Bridge added successfully"
                    }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_delete_bridge(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Path comes as "bridges/2" from route "/bridges/:bridge_id"
    let bridge_id_str = path.trim_start_matches('/').trim_start_matches("bridges/");
    let bridge_id = match bridge_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid bridge ID"),
    };

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.execute("DELETE FROM hue_bridges WHERE id = ?1", rusqlite::params![bridge_id]) {
                Ok(rows) => {
                    if rows > 0 {
                        json_response(&serde_json::json!({ "success": true, "message": "Bridge deleted" }))
                    } else {
                        error_response(StatusCode::NOT_FOUND, "Bridge not found")
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_pair_bridge(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Path comes as "bridges/2/pair" from route "/bridges/:bridge_id/pair"
    let bridge_id_str = path.trim_start_matches('/').trim_start_matches("bridges/").trim_end_matches("/pair");
    let bridge_id = match bridge_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid bridge ID"),
    };

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get bridge IP
    let ip_address: String = match conn.query_row(
        "SELECT ip_address FROM hue_bridges WHERE id = ?1",
        rusqlite::params![bridge_id],
        |row| row.get(0),
    ) {
        Ok(ip) => ip,
        Err(_) => return error_response(StatusCode::NOT_FOUND, "Bridge not found"),
    };

    // Make pairing request to Hue Bridge
    let client = reqwest::Client::new();
    let pair_url = format!("http://{}/api", ip_address);
    let pair_body = serde_json::json!({
        "devicetype": "webarcade#bridge"
    });

    match client.post(&pair_url).json(&pair_body).send().await {
        Ok(response) => {
            match response.json::<Vec<serde_json::Value>>().await {
                Ok(result) => {
                    if let Some(first) = result.first() {
                        if let Some(success) = first.get("success") {
                            if let Some(username) = success.get("username").and_then(|u| u.as_str()) {
                                // Store username
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64;

                                match conn.execute(
                                    "UPDATE hue_bridges SET username = ?1, last_connected = ?2 WHERE id = ?3",
                                    rusqlite::params![username, now, bridge_id],
                                ) {
                                    Ok(_) => {
                                        return json_response(&serde_json::json!({
                                            "success": true,
                                            "username": username,
                                            "message": "Bridge paired successfully"
                                        }));
                                    }
                                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                                }
                            }
                        }
                        if let Some(error) = first.get("error") {
                            if let Some(description) = error.get("description").and_then(|d| d.as_str()) {
                                return json_response(&serde_json::json!({
                                    "success": false,
                                    "message": description
                                }));
                            }
                        }
                    }
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Unexpected response from bridge")
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to connect to bridge: {}", e)),
    }
}

async fn handle_discover_lights(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Path comes as "bridges/2/discover" from route "/bridges/:bridge_id/discover"
    let bridge_id_str = path.trim_start_matches('/').trim_start_matches("bridges/").trim_end_matches("/discover");
    let bridge_id = match bridge_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid bridge ID"),
    };

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get bridge info
    let (ip_address, username): (String, Option<String>) = match conn.query_row(
        "SELECT ip_address, username FROM hue_bridges WHERE id = ?1",
        rusqlite::params![bridge_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ) {
        Ok(data) => data,
        Err(_) => return error_response(StatusCode::NOT_FOUND, "Bridge not found"),
    };

    let username = match username {
        Some(u) => u,
        None => return error_response(StatusCode::UNAUTHORIZED, "Bridge not paired"),
    };

    // Fetch lights from bridge
    let client = reqwest::Client::new();
    let lights_url = format!("http://{}/api/{}/lights", ip_address, username);

    match client.get(&lights_url).send().await {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(lights_data) => {
                    if let Some(lights_obj) = lights_data.as_object() {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;

                        // Store lights in database
                        for (light_id, light_data) in lights_obj {
                            if let Some(state) = light_data.get("state") {
                                let _ = conn.execute(
                                    "INSERT OR REPLACE INTO hue_lights (id, bridge_id, name, type, state_on, brightness, hue, saturation, color_temp, reachable, last_updated) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                                    rusqlite::params![
                                        light_id,
                                        bridge_id,
                                        light_data.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown"),
                                        light_data.get("type").and_then(|t| t.as_str()).unwrap_or("Unknown"),
                                        state.get("on").and_then(|o| o.as_bool()).unwrap_or(false) as i32,
                                        state.get("bri").and_then(|b| b.as_u64()).map(|b| b as i64),
                                        state.get("hue").and_then(|h| h.as_u64()).map(|h| h as i64),
                                        state.get("sat").and_then(|s| s.as_u64()).map(|s| s as i64),
                                        state.get("ct").and_then(|c| c.as_u64()).map(|c| c as i64),
                                        state.get("reachable").and_then(|r| r.as_bool()).unwrap_or(false) as i32,
                                        now,
                                    ],
                                );
                            }
                        }

                        // Fetch groups
                        let groups_url = format!("http://{}/api/{}/groups", ip_address, username);
                        if let Ok(groups_response) = client.get(&groups_url).send().await {
                            if let Ok(groups_data) = groups_response.json::<serde_json::Value>().await {
                                if let Some(groups_obj) = groups_data.as_object() {
                                    for (group_id, group_data) in groups_obj {
                                        if let Some(action) = group_data.get("action") {
                                            let lights_array = group_data.get("lights")
                                                .and_then(|l| l.as_array())
                                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
                                                .unwrap_or_default();

                                            let _ = conn.execute(
                                                "INSERT OR REPLACE INTO hue_groups (id, bridge_id, name, type, lights, state_on, brightness, last_updated) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                                                rusqlite::params![
                                                    group_id,
                                                    bridge_id,
                                                    group_data.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown"),
                                                    group_data.get("type").and_then(|t| t.as_str()).unwrap_or("Unknown"),
                                                    lights_array,
                                                    action.get("on").and_then(|o| o.as_bool()).unwrap_or(false) as i32,
                                                    action.get("bri").and_then(|b| b.as_u64()).map(|b| b as i64),
                                                    now,
                                                ],
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        json_response(&serde_json::json!({
                            "success": true,
                            "message": "Lights discovered and stored",
                            "count": lights_obj.len()
                        }))
                    } else {
                        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Invalid response format")
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch lights: {}", e)),
    }
}

async fn handle_get_lights() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare("SELECT id, bridge_id, name, type, state_on, brightness, hue, saturation, color_temp, reachable FROM hue_lights ORDER BY name") {
                Ok(s) => s,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let lights: Vec<serde_json::Value> = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "bridge_id": row.get::<_, i64>(1)?,
                    "name": row.get::<_, String>(2)?,
                    "type": row.get::<_, String>(3)?,
                    "state": {
                        "on": row.get::<_, i32>(4)? != 0,
                        "bri": row.get::<_, Option<i64>>(5)?,
                        "hue": row.get::<_, Option<i64>>(6)?,
                        "sat": row.get::<_, Option<i64>>(7)?,
                        "ct": row.get::<_, Option<i64>>(8)?,
                    },
                    "reachable": row.get::<_, i32>(9)? != 0,
                }))
            }).and_then(|mapped| mapped.collect()) {
                Ok(l) => l,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            json_response(&serde_json::json!({ "lights": lights }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_light(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Path comes as "lights/1" from route "/lights/:light_id"
    let light_id = path.trim_start_matches('/').trim_start_matches("lights/");

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.query_row(
                "SELECT id, bridge_id, name, type, state_on, brightness, hue, saturation, color_temp, reachable FROM hue_lights WHERE id = ?1",
                rusqlite::params![light_id],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, String>(0)?,
                        "bridge_id": row.get::<_, i64>(1)?,
                        "name": row.get::<_, String>(2)?,
                        "type": row.get::<_, String>(3)?,
                        "state": {
                            "on": row.get::<_, i32>(4)? != 0,
                            "bri": row.get::<_, Option<i64>>(5)?,
                            "hue": row.get::<_, Option<i64>>(6)?,
                            "sat": row.get::<_, Option<i64>>(7)?,
                            "ct": row.get::<_, Option<i64>>(8)?,
                        },
                        "reachable": row.get::<_, i32>(9)? != 0,
                    }))
                },
            ) {
                Ok(light) => json_response(&light),
                Err(_) => error_response(StatusCode::NOT_FOUND, "Light not found"),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_set_light_state(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Path comes as "lights/1/state" from route "/lights/:light_id/state"
    let light_id = path.trim_start_matches('/').trim_start_matches("lights/").trim_end_matches("/state");

    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let state: LightState = match serde_json::from_value(body) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get bridge info for this light
    let (bridge_id, ip_address, username): (i64, String, Option<String>) = match conn.query_row(
        "SELECT l.bridge_id, b.ip_address, b.username FROM hue_lights l JOIN hue_bridges b ON l.bridge_id = b.id WHERE l.id = ?1",
        rusqlite::params![light_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ) {
        Ok(data) => data,
        Err(_) => return error_response(StatusCode::NOT_FOUND, "Light not found"),
    };

    let username = match username {
        Some(u) => u,
        None => return error_response(StatusCode::UNAUTHORIZED, "Bridge not paired"),
    };

    // Send state update to bridge
    let client = reqwest::Client::new();
    let state_url = format!("http://{}/api/{}/lights/{}/state", ip_address, username, light_id);

    match client.put(&state_url).json(&state).send().await {
        Ok(response) => {
            match response.json::<Vec<serde_json::Value>>().await {
                Ok(_) => {
                    // Update local database
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    let _ = conn.execute(
                        "UPDATE hue_lights SET state_on = ?1, brightness = ?2, hue = ?3, saturation = ?4, color_temp = ?5, last_updated = ?6 WHERE id = ?7",
                        rusqlite::params![
                            state.on as i32,
                            state.bri.map(|b| b as i64),
                            state.hue.map(|h| h as i64),
                            state.sat.map(|s| s as i64),
                            state.ct.map(|c| c as i64),
                            now,
                            light_id,
                        ],
                    );

                    json_response(&serde_json::json!({
                        "success": true,
                        "message": "Light state updated"
                    }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update light: {}", e)),
    }
}

async fn handle_get_groups() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare("SELECT id, bridge_id, name, type, lights, state_on, brightness FROM hue_groups ORDER BY name") {
                Ok(s) => s,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let groups: Vec<serde_json::Value> = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "bridge_id": row.get::<_, i64>(1)?,
                    "name": row.get::<_, String>(2)?,
                    "type": row.get::<_, String>(3)?,
                    "lights": row.get::<_, String>(4)?.split(',').collect::<Vec<_>>(),
                    "state": {
                        "on": row.get::<_, i32>(5)? != 0,
                        "bri": row.get::<_, Option<i64>>(6)?,
                    },
                }))
            }).and_then(|mapped| mapped.collect()) {
                Ok(g) => g,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            json_response(&serde_json::json!({ "groups": groups }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_set_group_state(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Path comes as "groups/1/state" from route "/groups/:group_id/state"
    let group_id = path.trim_start_matches('/').trim_start_matches("groups/").trim_end_matches("/state");

    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let state: LightState = match serde_json::from_value(body) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get bridge info for this group
    let (bridge_id, ip_address, username): (i64, String, Option<String>) = match conn.query_row(
        "SELECT g.bridge_id, b.ip_address, b.username FROM hue_groups g JOIN hue_bridges b ON g.bridge_id = b.id WHERE g.id = ?1",
        rusqlite::params![group_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ) {
        Ok(data) => data,
        Err(_) => return error_response(StatusCode::NOT_FOUND, "Group not found"),
    };

    let username = match username {
        Some(u) => u,
        None => return error_response(StatusCode::UNAUTHORIZED, "Bridge not paired"),
    };

    // Send state update to bridge
    let client = reqwest::Client::new();
    let state_url = format!("http://{}/api/{}/groups/{}/action", ip_address, username, group_id);

    match client.put(&state_url).json(&state).send().await {
        Ok(response) => {
            match response.json::<Vec<serde_json::Value>>().await {
                Ok(_) => {
                    // Update local database
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    let _ = conn.execute(
                        "UPDATE hue_groups SET state_on = ?1, brightness = ?2, last_updated = ?3 WHERE id = ?4",
                        rusqlite::params![
                            state.on as i32,
                            state.bri.map(|b| b as i64),
                            now,
                            group_id,
                        ],
                    );

                    json_response(&serde_json::json!({
                        "success": true,
                        "message": "Group state updated"
                    }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update group: {}", e)),
    }
}

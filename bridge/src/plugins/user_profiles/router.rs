use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /user_profiles/profile/:user_id - Get user profile
    router.route(Method::GET, "/profile/:user_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_profile(path).await
        })
    });

    // POST /user_profiles/update - Create or update user profile
    router.route(Method::POST, "/update", |_path, _query, req| {
        Box::pin(async move {
            handle_update_profile(req).await
        })
    });

    // POST /user_profiles/field - Set custom field for user
    router.route(Method::POST, "/field", |_path, _query, req| {
        Box::pin(async move {
            handle_set_custom_field(req).await
        })
    });

    // GET /user_profiles/birthdays - Get today's birthdays
    router.route(Method::GET, "/birthdays", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_todays_birthdays().await
        })
    });

    // GET /user_profiles/search - Search profiles
    router.route(Method::GET, "/search", |_path, query, _req| {
        Box::pin(async move {
            handle_search_profiles(query).await
        })
    });

    ctx.register_router("user_profiles", router).await;
    Ok(())
}

async fn handle_get_profile(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/profile/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            use rusqlite::OptionalExtension;

            let profile: Option<serde_json::Value> = match conn.query_row(
                "SELECT user_id, username, display_name, birthday, location, timezone, bio, pronouns, social_links, created_at, updated_at
                 FROM user_profiles WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| {
                    Ok(serde_json::json!({
                        "user_id": row.get::<_, String>(0)?,
                        "username": row.get::<_, String>(1)?,
                        "display_name": row.get::<_, Option<String>>(2)?,
                        "birthday": row.get::<_, Option<String>>(3)?,
                        "location": row.get::<_, Option<String>>(4)?,
                        "timezone": row.get::<_, Option<String>>(5)?,
                        "bio": row.get::<_, Option<String>>(6)?,
                        "pronouns": row.get::<_, Option<String>>(7)?,
                        "social_links": row.get::<_, Option<String>>(8)?,
                        "created_at": row.get::<_, i64>(9)?,
                        "updated_at": row.get::<_, i64>(10)?,
                    }))
                }
            ).optional() {
                Ok(profile) => profile,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            // Get custom fields
            let mut custom_fields = serde_json::Map::new();
            if profile.is_some() {
                let mut stmt = match conn.prepare(
                    "SELECT field_name, field_value FROM user_profile_fields WHERE user_id = ?1"
                ) {
                    Ok(stmt) => stmt,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };

                let mapped = match stmt.query_map(rusqlite::params![user_id], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                }) {
                    Ok(m) => m,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };

                let fields: Result<Vec<_>, _> = mapped.collect();

                match fields {
                    Ok(fields) => {
                        for (name, value) in fields {
                            custom_fields.insert(name, serde_json::Value::String(value));
                        }
                    }
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                }
            }

            json_response(&serde_json::json!({
                "profile": profile,
                "custom_fields": custom_fields
            }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_profile(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
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

            let display_name = body.get("display_name").and_then(|v| v.as_str());
            let birthday = body.get("birthday").and_then(|v| v.as_str());
            let location = body.get("location").and_then(|v| v.as_str());
            let timezone = body.get("timezone").and_then(|v| v.as_str());
            let bio = body.get("bio").and_then(|v| v.as_str());
            let pronouns = body.get("pronouns").and_then(|v| v.as_str());
            let social_links = body.get("social_links").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    // Check if profile exists
                    let exists: bool = match conn.query_row(
                        "SELECT COUNT(*) FROM user_profiles WHERE user_id = ?1",
                        rusqlite::params![user_id],
                        |row| row.get::<_, i64>(0).map(|count| count > 0),
                    ) {
                        Ok(exists) => exists,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    if exists {
                        // Update existing profile
                        match conn.execute(
                            "UPDATE user_profiles SET username = ?1, display_name = ?2, birthday = ?3,
                             location = ?4, timezone = ?5, bio = ?6, pronouns = ?7, social_links = ?8, updated_at = ?9
                             WHERE user_id = ?10",
                            rusqlite::params![username, display_name, birthday, location, timezone, bio, pronouns, social_links, now, user_id],
                        ) {
                            Ok(_) => {},
                            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                        }
                    } else {
                        // Create new profile
                        match conn.execute(
                            "INSERT INTO user_profiles (user_id, username, display_name, birthday, location, timezone, bio, pronouns, social_links, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                            rusqlite::params![user_id, username, display_name, birthday, location, timezone, bio, pronouns, social_links, now],
                        ) {
                            Ok(_) => {},
                            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                        }
                    }

                    // If birthday was set, update birthday reminder
                    if let Some(bday) = birthday {
                        match conn.execute(
                            "INSERT OR REPLACE INTO birthday_reminders (user_id, username, birthday, created_at)
                             VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![user_id, username, bday, now],
                        ) {
                            Ok(_) => {},
                            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                        }
                    }

                    json_response(&serde_json::json!({ "success": true }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_set_custom_field(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let field_name = match body.get("field_name").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing field_name"),
            };
            let field_value = match body.get("field_value").and_then(|v| v.as_str()) {
                Some(value) => value,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing field_value"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    match conn.execute(
                        "INSERT OR REPLACE INTO user_profile_fields (user_id, field_name, field_value, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?4)",
                        rusqlite::params![user_id, field_name, field_value, now],
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

async fn handle_get_todays_birthdays() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Get current date in MM-DD format
            let now = chrono::Utc::now();
            let today = now.format("%m-%d").to_string();

            let mut stmt = match conn.prepare(
                "SELECT user_id, username, birthday FROM birthday_reminders WHERE substr(birthday, 6, 5) = ?1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(rusqlite::params![today], |row| {
                Ok(serde_json::json!({
                    "user_id": row.get::<_, String>(0)?,
                    "username": row.get::<_, String>(1)?,
                    "birthday": row.get::<_, String>(2)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let birthdays: Result<Vec<_>, _> = mapped.collect();

            match birthdays {
                Ok(birthdays) => json_response(&serde_json::json!({ "birthdays": birthdays })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_search_profiles(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let search_query = match parse_query_param(&query, "query") {
        Some(q) => q,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing query parameter"),
    };

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let search_pattern = format!("%{}%", search_query);
            let mut stmt = match conn.prepare(
                "SELECT user_id, username, display_name, location, bio
                 FROM user_profiles
                 WHERE username LIKE ?1 OR display_name LIKE ?1 OR location LIKE ?1
                 LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(
                rusqlite::params![search_pattern, limit],
                |row| {
                    Ok(serde_json::json!({
                        "user_id": row.get::<_, String>(0)?,
                        "username": row.get::<_, String>(1)?,
                        "display_name": row.get::<_, Option<String>>(2)?,
                        "location": row.get::<_, Option<String>>(3)?,
                        "bio": row.get::<_, Option<String>>(4)?,
                    }))
                }
            ) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let profiles: Result<Vec<_>, _> = mapped.collect();

            match profiles {
                Ok(profiles) => json_response(&serde_json::json!({ "profiles": profiles })),
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

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

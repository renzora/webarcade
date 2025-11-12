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
struct Dashboard {
    id: String,
    name: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct WidgetInstance {
    id: String,
    dashboard_id: String,
    widget_id: String,
    order_index: i32,
    columns: i32,
    config: Option<String>,
    created_at: i64,
    updated_at: i64,
}

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // Dashboard routes
    route!(router, GET "/dashboards" => handle_get_dashboards);
    route!(router, POST "/dashboards" => handle_create_dashboard);
    route!(router, PUT "/dashboards/:id", path => handle_update_dashboard);
    route!(router, DELETE "/dashboards/:id", path => handle_delete_dashboard);

    // Widget instance routes
    route!(router, GET "/dashboards/:id/widgets", path => handle_get_widgets);
    route!(router, POST "/dashboards/:id/widgets", path => handle_create_widget);
    route!(router, PUT "/widgets/:id", path => handle_update_widget);
    route!(router, DELETE "/widgets/:id", path => handle_delete_widget);
    route!(router, POST "/dashboards/:id/widgets/reorder", path => handle_reorder_widgets);

    ctx.register_router("dashboard", router).await;
    Ok(())
}

// GET /dashboard/dashboards
async fn handle_get_dashboards() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.prepare("SELECT id, name, created_at, updated_at FROM dashboards ORDER BY created_at ASC") {
                Ok(mut stmt) => {
                    let dashboards_iter = stmt.query_map([], |row| {
                        Ok(Dashboard {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            created_at: row.get(2)?,
                            updated_at: row.get(3)?,
                        })
                    });

                    match dashboards_iter {
                        Ok(iter) => {
                            let dashboards: Vec<Dashboard> = iter.filter_map(|r| r.ok()).collect();
                            json_response(&dashboards)
                        }
                        Err(e) => {
                            log::error!("[Dashboard] Query error: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to query dashboards")
                        }
                    }
                }
                Err(e) => {
                    log::error!("[Dashboard] Prepare error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to prepare query")
                }
            }
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// POST /dashboard/dashboards
async fn handle_create_dashboard(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let name = match body.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing name parameter"),
    };

    let now = chrono::Utc::now().timestamp();
    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let id = format!("dashboard_{}_{}", now, nanos % 1000000);

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.execute(
                "INSERT INTO dashboards (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
                rusqlite::params![&id, name, now, now],
            ) {
                Ok(_) => {
                    let dashboard = Dashboard {
                        id,
                        name: name.to_string(),
                        created_at: now,
                        updated_at: now,
                    };
                    json_response(&dashboard)
                }
                Err(e) => {
                    log::error!("[Dashboard] Insert error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create dashboard")
                }
            }
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// PUT /dashboard/dashboards/:id
async fn handle_update_dashboard(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract id from path (format: "/dashboards/ID")
    let id = path.trim_start_matches('/').trim_start_matches("dashboards/");

    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let name = match body.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing name parameter"),
    };

    let now = chrono::Utc::now().timestamp();

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.execute(
                "UPDATE dashboards SET name = ?, updated_at = ? WHERE id = ?",
                rusqlite::params![name, now, id],
            ) {
                Ok(_) => json_response(&serde_json::json!({"success": true})),
                Err(e) => {
                    log::error!("[Dashboard] Update error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update dashboard")
                }
            }
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// DELETE /dashboard/dashboards/:id
async fn handle_delete_dashboard(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id = path.trim_start_matches('/').trim_start_matches("dashboards/");

    if id == "default" {
        return error_response(StatusCode::BAD_REQUEST, "Cannot delete the default dashboard");
    }

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.execute("DELETE FROM dashboards WHERE id = ?", rusqlite::params![id]) {
                Ok(_) => json_response(&serde_json::json!({"success": true})),
                Err(e) => {
                    log::error!("[Dashboard] Delete error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete dashboard")
                }
            }
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// GET /dashboard/dashboards/:id/widgets
async fn handle_get_widgets(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract id from path (format: "/dashboards/ID/widgets")
    let dashboard_id = path.trim_start_matches('/').trim_start_matches("dashboards/").trim_end_matches("/widgets");

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.prepare(
                "SELECT id, dashboard_id, widget_id, order_index, columns, config, created_at, updated_at
                 FROM widget_instances WHERE dashboard_id = ? ORDER BY order_index ASC"
            ) {
                Ok(mut stmt) => {
                    let widgets_iter = stmt.query_map(rusqlite::params![dashboard_id], |row| {
                        Ok(WidgetInstance {
                            id: row.get(0)?,
                            dashboard_id: row.get(1)?,
                            widget_id: row.get(2)?,
                            order_index: row.get(3)?,
                            columns: row.get(4)?,
                            config: row.get(5).ok(),
                            created_at: row.get(6)?,
                            updated_at: row.get(7)?,
                        })
                    });

                    match widgets_iter {
                        Ok(iter) => {
                            let widgets: Vec<WidgetInstance> = iter.filter_map(|r| r.ok()).collect();
                            json_response(&widgets)
                        }
                        Err(e) => {
                            log::error!("[Dashboard] Query error: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to query widgets")
                        }
                    }
                }
                Err(e) => {
                    log::error!("[Dashboard] Prepare error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to prepare query")
                }
            }
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// POST /dashboard/dashboards/:id/widgets
async fn handle_create_widget(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract id from path (format: "/dashboards/ID/widgets")
    let dashboard_id = path.trim_start_matches('/').trim_start_matches("dashboards/").trim_end_matches("/widgets").to_string();
    log::info!("[Dashboard] Creating widget for dashboard: '{}'", dashboard_id);

    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let widget_id = match body.get("widget_id").and_then(|v| v.as_str()) {
        Some(w) => w,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing widget_id parameter"),
    };

    let order_index = body.get("order_index").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let columns = body.get("columns").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    let config = body.get("config").and_then(|v| v.as_str()).map(|s| s.to_string());

    let now = chrono::Utc::now().timestamp();
    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let id = format!("widget_{}_{}", now, nanos % 1000000);

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.execute(
                "INSERT INTO widget_instances (id, dashboard_id, widget_id, order_index, columns, config, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![&id, &dashboard_id, widget_id, order_index, columns, config.as_deref().unwrap_or(""), now, now],
            ) {
                Ok(_) => {
                    let widget = WidgetInstance {
                        id,
                        dashboard_id,
                        widget_id: widget_id.to_string(),
                        order_index,
                        columns,
                        config,
                        created_at: now,
                        updated_at: now,
                    };
                    json_response(&widget)
                }
                Err(e) => {
                    log::error!("[Dashboard] Insert error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create widget")
                }
            }
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// PUT /dashboard/widgets/:id
async fn handle_update_widget(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let id = path.trim_start_matches('/').trim_start_matches("widgets/");

    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let now = chrono::Utc::now().timestamp();
    let db_path = crate::core::database::get_database_path();

    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            // Build update query dynamically based on provided fields
            if let Some(order_index) = body.get("order_index").and_then(|v| v.as_i64()) {
                let _ = conn.execute(
                    "UPDATE widget_instances SET order_index = ?, updated_at = ? WHERE id = ?",
                    rusqlite::params![order_index, now, id],
                );
            }
            if let Some(columns) = body.get("columns").and_then(|v| v.as_i64()) {
                let _ = conn.execute(
                    "UPDATE widget_instances SET columns = ?, updated_at = ? WHERE id = ?",
                    rusqlite::params![columns, now, id],
                );
            }
            if let Some(config) = body.get("config").and_then(|v| v.as_str()) {
                let _ = conn.execute(
                    "UPDATE widget_instances SET config = ?, updated_at = ? WHERE id = ?",
                    rusqlite::params![config, now, id],
                );
            }

            json_response(&serde_json::json!({"success": true}))
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// DELETE /dashboard/widgets/:id
async fn handle_delete_widget(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id = path.trim_start_matches('/').trim_start_matches("widgets/");

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.execute("DELETE FROM widget_instances WHERE id = ?", rusqlite::params![id]) {
                Ok(_) => json_response(&serde_json::json!({"success": true})),
                Err(e) => {
                    log::error!("[Dashboard] Delete error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete widget")
                }
            }
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

// POST /dashboard/dashboards/:id/widgets/reorder
async fn handle_reorder_widgets(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract id from path (format: "/dashboards/ID/widgets/reorder")
    let dashboard_id = path.trim_start_matches('/').trim_start_matches("dashboards/").trim_end_matches("/widgets/reorder");

    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let widget_ids = match body.get("widget_ids").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing widget_ids parameter"),
    };

    let now = chrono::Utc::now().timestamp();
    let db_path = crate::core::database::get_database_path();

    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            for (index, widget_id_val) in widget_ids.iter().enumerate() {
                if let Some(widget_id) = widget_id_val.as_str() {
                    let _ = conn.execute(
                        "UPDATE widget_instances SET order_index = ?, updated_at = ? WHERE id = ? AND dashboard_id = ?",
                        rusqlite::params![index as i32, now, widget_id, dashboard_id],
                    );
                }
            }

            json_response(&serde_json::json!({"success": true}))
        }
        Err(e) => {
            log::error!("[Dashboard] Database connection error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed")
        }
    }
}

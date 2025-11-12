use webarcade_api::prelude::*;

#[derive(Serialize, Deserialize)]
struct SystemStats {
    cpu_usage: f32,
    cpu_count: usize,
    ram_used: u64,
    ram_total: u64,
    ram_percent: f32,
}

#[derive(Serialize, Deserialize)]
struct HistoricalStats {
    cpu_usage: f64,
    ram_used: i64,
    ram_total: i64,
    timestamp: i64,
}

pub async fn register_routes(ctx: &Context) -> Result<()> {
    let mut router = Router::new();

    route!(router, GET "/stats" => handle_stats);
    route!(router, GET "/history" => handle_history);
    route!(router, POST "/save-stats" => handle_save_stats);

    ctx.register_router("sysmonitor", router).await;

    Ok(())
}

async fn handle_stats() -> HttpResponse {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let cpu_count = sys.cpus().len();
    let ram_used = sys.used_memory();
    let ram_total = sys.total_memory();
    let ram_percent = (ram_used as f32 / ram_total as f32) * 100.0;

    let stats = SystemStats {
        cpu_usage,
        cpu_count,
        ram_used,
        ram_total,
        ram_percent,
    };

    json_response(&stats)
}

async fn handle_history() -> HttpResponse {
    let ctx = Context::global();
    let db = ctx.db();

    match db.query::<HistoricalStats>(
        "SELECT cpu_usage, ram_used, ram_total, timestamp FROM system_stats ORDER BY timestamp DESC LIMIT 100",
        &json!([])
    ) {
        Ok(stats) => json_response(&stats),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to fetch history: {}", e)
        )
    }
}

async fn handle_save_stats() -> HttpResponse {
    use sysinfo::System;

    let ctx = Context::global();
    let db = ctx.db();

    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
    let ram_used = sys.used_memory() as i64;
    let ram_total = sys.total_memory() as i64;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    match db.execute(
        "INSERT INTO system_stats (cpu_usage, ram_used, ram_total, timestamp) VALUES (?, ?, ?, ?)",
        &json!([cpu_usage, ram_used, ram_total, timestamp])
    ) {
        Ok(_) => json_response(&json!({ "success": true })),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to save stats: {}", e)
        )
    }
}

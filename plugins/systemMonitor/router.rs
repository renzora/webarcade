use api::{HttpResponse, json, json_response};
use sysinfo::System;
use nvml_wrapper::Nvml;

fn get_gpu_stats() -> Option<(f64, u64, u64, String)> {
    let nvml = Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;

    let utilization = device.utilization_rates().ok()?;
    let memory_info = device.memory_info().ok()?;
    let name = device.name().unwrap_or_else(|_| "Unknown GPU".to_string());

    let gpu_usage = utilization.gpu as f64;
    let memory_used = memory_info.used;
    let memory_total = memory_info.total;

    Some((gpu_usage, memory_used, memory_total, name))
}

pub async fn handle_stats() -> HttpResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_count = sys.cpus().len();
    let cpu_usage: f64 = sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64).sum::<f64>() / cpu_count as f64;
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;

    let gpu_stats = get_gpu_stats();
    let has_gpu = gpu_stats.is_some();
    let (gpu_usage, gpu_memory_used, gpu_memory_total, gpu_name) = gpu_stats.unwrap_or((0.0, 0, 0, String::new()));
    let gpu_memory_usage = if gpu_memory_total > 0 {
        (gpu_memory_used as f64 / gpu_memory_total as f64) * 100.0
    } else {
        0.0
    };

    let response = json!({
        "cpu_usage": cpu_usage,
        "memory_usage": memory_usage,
        "gpu_usage": if has_gpu { Some(gpu_usage) } else { None::<f64> },
        "cpu": {
            "cores": cpu_count,
            "usage_percent": cpu_usage,
        },
        "memory": {
            "total": total_memory,
            "used": used_memory,
            "usage_percent": memory_usage,
        },
        "gpu": if has_gpu {
            Some(json!({
                "name": gpu_name,
                "usage_percent": gpu_usage,
                "memory_total": gpu_memory_total,
                "memory_used": gpu_memory_used,
                "memory_usage_percent": gpu_memory_usage,
            }))
        } else {
            None
        },
    });

    json_response(&response)
}

pub async fn handle_cpu() -> HttpResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_count = sys.cpus().len();
    let cpu_usage: f64 = sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64).sum::<f64>() / cpu_count as f64;

    let response = json!({
        "cores": cpu_count,
        "usage_percent": cpu_usage,
    });

    json_response(&response)
}

pub async fn handle_memory() -> HttpResponse {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;

    let response = json!({
        "total": total_memory,
        "used": used_memory,
        "usage_percent": memory_usage,
    });

    json_response(&response)
}

pub async fn handle_gpu() -> HttpResponse {
    match get_gpu_stats() {
        Some((gpu_usage, memory_used, memory_total, name)) => {
            let memory_usage = if memory_total > 0 {
                (memory_used as f64 / memory_total as f64) * 100.0
            } else {
                0.0
            };

            let response = json!({
                "available": true,
                "name": name,
                "usage_percent": gpu_usage,
                "memory_total": memory_total,
                "memory_used": memory_used,
                "memory_usage_percent": memory_usage,
            });

            json_response(&response)
        }
        None => {
            let response = json!({
                "available": false,
                "error": "No NVIDIA GPU found or NVML not available"
            });

            json_response(&response)
        }
    }
}

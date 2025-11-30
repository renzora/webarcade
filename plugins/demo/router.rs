use api::{HttpRequest, HttpResponse, json, json_response, serde_json};
use notify_rust::Notification;
use serde::Deserialize;
use sysinfo::System;
use nvml_wrapper::Nvml;
use brightness::Brightness;

pub async fn handle_hello() -> HttpResponse {
    println!("[Demo Plugin] Hello from Rust backend!");

    let response = json!({
        "message": "Hello from the Demo Plugin Rust backend!",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    json_response(&response)
}

#[derive(Deserialize)]
struct NotifyRequest {
    title: Option<String>,
    message: Option<String>,
}

pub async fn handle_notify(req: HttpRequest) -> HttpResponse {
    let body: NotifyRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "success": false,
                "error": format!("Invalid JSON body: {}", e)
            }));
        }
    };

    let title = body.title.unwrap_or_else(|| "Demo Plugin".to_string());
    let message = body.message.unwrap_or_else(|| "Hello from WebArcade!".to_string());

    println!("[Demo Plugin] Sending notification: {} - {}", title, message);

    match Notification::new()
        .summary(&title)
        .body(&message)
        .timeout(5000)
        .show()
    {
        Ok(_) => {
            json_response(&json!({
                "success": true,
                "message": "Notification sent successfully!"
            }))
        }
        Err(e) => {
            json_response(&json!({
                "success": false,
                "error": format!("Failed to send notification: {}", e)
            }))
        }
    }
}

pub async fn handle_cpu_info() -> HttpResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpuid = raw_cpuid::CpuId::new();

    let brand = cpuid.get_processor_brand_string()
        .map(|b| b.as_str().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let features = cpuid.get_feature_info();

    let cpu_count = sys.cpus().len();
    let physical_cores = sys.physical_core_count().unwrap_or(cpu_count);

    // Get CPU usage
    let cpu_usage: f64 = sys.cpus().iter()
        .map(|cpu| cpu.cpu_usage() as f64)
        .sum::<f64>() / cpu_count as f64;

    // Get per-core usage
    let per_core_usage: Vec<f64> = sys.cpus().iter()
        .map(|cpu| cpu.cpu_usage() as f64)
        .collect();

    // Get frequency
    let frequency = sys.cpus().first()
        .map(|cpu| cpu.frequency())
        .unwrap_or(0);

    let response = json!({
        "brand": brand.trim(),
        "physical_cores": physical_cores,
        "logical_cores": cpu_count,
        "frequency_mhz": frequency,
        "usage_percent": cpu_usage,
        "per_core_usage": per_core_usage,
        "architecture": std::env::consts::ARCH,
        "has_sse": features.as_ref().map(|f| f.has_sse()).unwrap_or(false),
        "has_sse2": features.as_ref().map(|f| f.has_sse2()).unwrap_or(false),
        "has_avx": features.as_ref().map(|f| f.has_avx()).unwrap_or(false),
    });

    json_response(&response)
}

pub async fn handle_gpu_info() -> HttpResponse {
    match Nvml::init() {
        Ok(nvml) => {
            match nvml.device_by_index(0) {
                Ok(device) => {
                    let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
                    let memory_info = device.memory_info().ok();
                    let utilization = device.utilization_rates().ok();
                    let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu).ok();
                    let power_usage = device.power_usage().ok();
                    let driver_version = nvml.sys_driver_version().unwrap_or_else(|_| "Unknown".to_string());
                    let cuda_version = nvml.sys_cuda_driver_version().ok();

                    let response = json!({
                        "available": true,
                        "name": name,
                        "driver_version": driver_version,
                        "cuda_version": cuda_version.map(|v| format!("{}.{}", v / 1000, (v % 1000) / 10)),
                        "memory": memory_info.map(|m| json!({
                            "total_mb": m.total / 1024 / 1024,
                            "used_mb": m.used / 1024 / 1024,
                            "free_mb": m.free / 1024 / 1024,
                            "usage_percent": (m.used as f64 / m.total as f64) * 100.0
                        })),
                        "utilization": utilization.map(|u| json!({
                            "gpu_percent": u.gpu,
                            "memory_percent": u.memory
                        })),
                        "temperature_c": temperature,
                        "power_watts": power_usage.map(|p| p as f64 / 1000.0),
                    });

                    json_response(&response)
                }
                Err(e) => {
                    json_response(&json!({
                        "available": false,
                        "error": format!("Failed to get GPU: {}", e)
                    }))
                }
            }
        }
        Err(_) => {
            json_response(&json!({
                "available": false,
                "error": "No NVIDIA GPU found or NVML not available"
            }))
        }
    }
}

pub async fn handle_ram_info() -> HttpResponse {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    let response = json!({
        "total_gb": total as f64 / 1024.0 / 1024.0 / 1024.0,
        "used_gb": used as f64 / 1024.0 / 1024.0 / 1024.0,
        "available_gb": available as f64 / 1024.0 / 1024.0 / 1024.0,
        "usage_percent": (used as f64 / total as f64) * 100.0,
        "swap": {
            "total_gb": total_swap as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_gb": used_swap as f64 / 1024.0 / 1024.0 / 1024.0,
            "usage_percent": if total_swap > 0 { (used_swap as f64 / total_swap as f64) * 100.0 } else { 0.0 }
        }
    });

    json_response(&response)
}

pub async fn handle_usb_devices() -> HttpResponse {
    // Use WMI to query USB devices on Windows
    let devices = get_usb_devices_wmi();

    json_response(&json!({
        "devices": devices
    }))
}

fn get_usb_devices_wmi() -> Vec<serde_json::Value> {
    use wmi::{COMLibrary, WMIConnection};
    use std::collections::HashMap;

    let com_lib = match COMLibrary::new() {
        Ok(lib) => lib,
        Err(_) => return vec![],
    };

    let wmi_con = match WMIConnection::new(com_lib) {
        Ok(con) => con,
        Err(_) => return vec![],
    };

    let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
        .raw_query("SELECT * FROM Win32_USBHub")
        .unwrap_or_default();

    results.iter().map(|device| {
        json!({
            "name": device.get("Name").map(variant_to_string).unwrap_or_default(),
            "device_id": device.get("DeviceID").map(variant_to_string).unwrap_or_default(),
            "status": device.get("Status").map(variant_to_string).unwrap_or_default(),
        })
    }).collect()
}

fn variant_to_string(v: &wmi::Variant) -> String {
    match v {
        wmi::Variant::String(s) => s.clone(),
        wmi::Variant::I4(i) => i.to_string(),
        wmi::Variant::UI4(i) => i.to_string(),
        wmi::Variant::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

pub async fn handle_audio_level() -> HttpResponse {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    // Try to get current audio output level
    let host = cpal::default_host();

    // Get output device info
    let output_device = host.default_output_device();
    let input_device = host.default_input_device();

    let output_info = output_device.as_ref().map(|d| {
        json!({
            "name": d.name().unwrap_or_else(|_| "Unknown".to_string()),
        })
    });

    let input_info = input_device.as_ref().map(|d| {
        json!({
            "name": d.name().unwrap_or_else(|_| "Unknown".to_string()),
        })
    });

    // List all output devices
    let output_devices: Vec<serde_json::Value> = host.output_devices()
        .map(|devices| {
            devices.filter_map(|d| {
                Some(json!({
                    "name": d.name().ok()?
                }))
            }).collect()
        })
        .unwrap_or_default();

    // List all input devices
    let input_devices: Vec<serde_json::Value> = host.input_devices()
        .map(|devices| {
            devices.filter_map(|d| {
                Some(json!({
                    "name": d.name().ok()?
                }))
            }).collect()
        })
        .unwrap_or_default();

    let response = json!({
        "default_output": output_info,
        "default_input": input_info,
        "output_devices": output_devices,
        "input_devices": input_devices,
        "host": host.id().name(),
    });

    json_response(&response)
}

pub async fn handle_get_brightness() -> HttpResponse {
    use futures::StreamExt;

    let devices: Vec<_> = brightness::brightness_devices().collect().await;

    let mut monitors = Vec::new();
    for device in devices {
        if let Ok(dev) = device {
            let name = dev.device_name().await.unwrap_or_else(|_| "Unknown".to_string());
            let level = dev.get().await.unwrap_or(0);
            monitors.push(json!({
                "name": name,
                "brightness": level
            }));
        }
    }

    json_response(&json!({
        "monitors": monitors
    }))
}

#[derive(Deserialize)]
struct BrightnessRequest {
    brightness: u32,
    monitor: Option<String>,
}

pub async fn handle_set_brightness(req: HttpRequest) -> HttpResponse {
    use futures::StreamExt;

    let body: BrightnessRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "success": false,
                "error": format!("Invalid JSON body: {}", e)
            }));
        }
    };

    let brightness_value = body.brightness.min(100);
    let devices: Vec<_> = brightness::brightness_devices().collect().await;

    let mut success = false;
    let mut error_msg = String::new();

    for device in devices {
        if let Ok(mut dev) = device {
            let name = dev.device_name().await.unwrap_or_else(|_| "Unknown".to_string());

            // If a specific monitor is requested, only set that one
            if let Some(ref target) = body.monitor {
                if !name.contains(target) {
                    continue;
                }
            }

            match dev.set(brightness_value).await {
                Ok(_) => {
                    success = true;
                    println!("[Demo Plugin] Set brightness to {}% on {}", brightness_value, name);
                }
                Err(e) => {
                    error_msg = format!("Failed to set brightness: {}", e);
                }
            }
        }
    }

    if success {
        json_response(&json!({
            "success": true,
            "brightness": brightness_value
        }))
    } else {
        json_response(&json!({
            "success": false,
            "error": if error_msg.is_empty() { "No monitors found".to_string() } else { error_msg }
        }))
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgrokTunnel {
    pub public_url: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct NgrokTunnelsResponse {
    tunnels: Vec<NgrokTunnelInfo>,
}

#[derive(Debug, Deserialize)]
struct NgrokTunnelInfo {
    public_url: String,
    name: String,
}

pub struct NgrokManager {
    process: Arc<Mutex<Option<tokio::process::Child>>>,
}

impl NgrokManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let mut process_lock = self.process.lock().await;

        // Check if already running
        if let Some(child) = process_lock.as_mut() {
            // Check if process is still alive
            if child.try_wait()?.is_none() {
                log::info!("[Ngrok] Already running");
                return Ok(());
            }
        }

        log::info!("[Ngrok] Starting ngrok tunnel on port {}", port);

        // Start ngrok process
        let child = Command::new("ngrok")
            .arg("http")
            .arg(port.to_string())
            .arg("--log")
            .arg("stdout")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        *process_lock = Some(child);

        // Give ngrok time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        log::info!("[Ngrok] Tunnel started successfully");

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut process_lock = self.process.lock().await;

        if let Some(mut child) = process_lock.take() {
            log::info!("[Ngrok] Stopping ngrok tunnel");
            child.kill().await?;
            log::info!("[Ngrok] Tunnel stopped");
        }

        Ok(())
    }

    pub async fn get_public_url(&self) -> Result<String> {
        // Query ngrok API for tunnels
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:4040/api/tunnels")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Ngrok API not available. Is ngrok running?");
        }

        let tunnels: NgrokTunnelsResponse = response.json().await?;

        // Find the first HTTPS tunnel
        for tunnel in tunnels.tunnels {
            if tunnel.public_url.starts_with("https://") {
                return Ok(tunnel.public_url);
            }
        }

        anyhow::bail!("No HTTPS tunnel found")
    }

    pub async fn is_running(&self) -> bool {
        match self.get_public_url().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub async fn get_status(&self) -> Result<serde_json::Value> {
        let is_running = self.is_running().await;

        if is_running {
            let public_url = self.get_public_url().await?;
            Ok(serde_json::json!({
                "running": true,
                "public_url": public_url
            }))
        } else {
            Ok(serde_json::json!({
                "running": false
            }))
        }
    }
}

impl Drop for NgrokManager {
    fn drop(&mut self) {
        // Best effort cleanup
        if let Some(mut child) = self.process.try_lock().ok().and_then(|mut lock| lock.take()) {
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/T", "/PID", &child.id().unwrap().to_string()])
                .output();
        }
    }
}

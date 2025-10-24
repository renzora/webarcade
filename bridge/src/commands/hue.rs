use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HueLight {
    pub id: String,
    pub name: String,
    pub on: bool,
    pub brightness: Option<u8>,  // 1-254
    pub hue: Option<u16>,         // 0-65535
    pub saturation: Option<u8>,   // 0-254
    pub reachable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct HueLightState {
    on: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    bri: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hue: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sat: Option<u8>,
    reachable: bool,
}

#[derive(Debug, Deserialize)]
struct HueLightResponse {
    name: String,
    state: HueLightState,
}

#[derive(Debug, Clone)]
pub struct HueClient {
    bridge_ip: String,
    username: String,
    client: Client,
}

impl HueClient {
    pub fn new(bridge_ip: String, username: String) -> Self {
        Self {
            bridge_ip,
            username,
            client: Client::new(),
        }
    }

    fn get_base_url(&self) -> String {
        format!("http://{}/api/{}", self.bridge_ip, self.username)
    }

    /// Discover Hue bridge on the network
    pub async fn discover_bridge() -> Result<String, String> {
        let client = Client::new();

        match client.get("https://discovery.meethue.com/")
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(bridges) = response.json::<Vec<serde_json::Value>>().await {
                    if let Some(bridge) = bridges.first() {
                        if let Some(ip) = bridge.get("internalipaddress").and_then(|v| v.as_str()) {
                            return Ok(ip.to_string());
                        }
                    }
                }
                Err("No Hue bridge found on network".to_string())
            }
            Err(e) => Err(format!("Failed to discover bridge: {}", e)),
        }
    }

    /// Create a new user (requires bridge button press)
    pub async fn create_user(bridge_ip: &str, app_name: &str) -> Result<String, String> {
        let client = Client::new();
        let url = format!("http://{}/api", bridge_ip);

        let body = serde_json::json!({
            "devicetype": app_name
        });

        match client.post(&url)
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(results) = response.json::<Vec<serde_json::Value>>().await {
                    if let Some(result) = results.first() {
                        if let Some(success) = result.get("success") {
                            if let Some(username) = success.get("username").and_then(|v| v.as_str()) {
                                return Ok(username.to_string());
                            }
                        }
                        if let Some(error) = result.get("error") {
                            if let Some(description) = error.get("description").and_then(|v| v.as_str()) {
                                return Err(description.to_string());
                            }
                        }
                    }
                }
                Err("Unexpected response from bridge".to_string())
            }
            Err(e) => Err(format!("Failed to create user: {}", e)),
        }
    }

    /// Get all lights
    pub async fn get_lights(&self) -> Result<Vec<HueLight>, String> {
        let url = format!("{}/lights", self.get_base_url());

        match self.client.get(&url).send().await {
            Ok(response) => {
                if let Ok(lights_map) = response.json::<HashMap<String, HueLightResponse>>().await {
                    let lights: Vec<HueLight> = lights_map
                        .into_iter()
                        .map(|(id, light)| HueLight {
                            id,
                            name: light.name,
                            on: light.state.on,
                            brightness: light.state.bri,
                            hue: light.state.hue,
                            saturation: light.state.sat,
                            reachable: light.state.reachable,
                        })
                        .collect();
                    Ok(lights)
                } else {
                    Err("Failed to parse lights response".to_string())
                }
            }
            Err(e) => Err(format!("Failed to get lights: {}", e)),
        }
    }

    /// Turn light on/off
    pub async fn set_light_power(&self, light_id: &str, on: bool) -> Result<(), String> {
        let url = format!("{}/lights/{}/state", self.get_base_url(), light_id);

        let body = serde_json::json!({ "on": on });

        match self.client.put(&url).json(&body).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to set light power: {}", e)),
        }
    }

    /// Set light brightness (1-254)
    pub async fn set_light_brightness(&self, light_id: &str, brightness: u8) -> Result<(), String> {
        let brightness = brightness.max(1).min(254);
        let url = format!("{}/lights/{}/state", self.get_base_url(), light_id);

        let body = serde_json::json!({
            "on": true,
            "bri": brightness
        });

        match self.client.put(&url).json(&body).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to set brightness: {}", e)),
        }
    }

    /// Set light color (hue: 0-65535, saturation: 0-254, transitiontime in deciseconds)
    pub async fn set_light_color(&self, light_id: &str, hue: u16, saturation: u8) -> Result<(), String> {
        self.set_light_color_with_transition(light_id, hue, saturation, None).await
    }

    /// Set light color with custom transition time (in deciseconds: 1 = 0.1s, 10 = 1s)
    pub async fn set_light_color_with_transition(&self, light_id: &str, hue: u16, saturation: u8, transitiontime: Option<u16>) -> Result<(), String> {
        let saturation = saturation.min(254);
        let url = format!("{}/lights/{}/state", self.get_base_url(), light_id);

        let mut body = serde_json::json!({
            "on": true,
            "hue": hue,
            "sat": saturation
        });

        if let Some(tt) = transitiontime {
            body["transitiontime"] = serde_json::json!(tt);
        }

        match self.client.put(&url).json(&body).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to set color: {}", e)),
        }
    }

    /// Set light to RGB color (approximation)
    pub async fn set_light_rgb(&self, light_id: &str, r: u8, g: u8, b: u8) -> Result<(), String> {
        self.set_light_rgb_with_transition(light_id, r, g, b, None).await
    }

    /// Set light to RGB color with transition time (in deciseconds)
    pub async fn set_light_rgb_with_transition(&self, light_id: &str, r: u8, g: u8, b: u8, transitiontime: Option<u16>) -> Result<(), String> {
        let (hue, sat, _bri) = rgb_to_hsb(r, g, b);
        self.set_light_color_with_transition(light_id, hue, sat, transitiontime).await
    }

    /// Turn all lights on/off
    pub async fn set_all_lights(&self, on: bool) -> Result<(), String> {
        let lights = self.get_lights().await?;

        for light in lights {
            self.set_light_power(&light.id, on).await?;
        }

        Ok(())
    }

    /// Set a scene by name (red, blue, green, purple, etc.)
    pub async fn set_scene(&self, scene_name: &str) -> Result<(), String> {
        self.set_scene_with_transition(scene_name, None).await
    }

    /// Set a scene with custom transition time (in deciseconds)
    pub async fn set_scene_with_transition(&self, scene_name: &str, transitiontime: Option<u16>) -> Result<(), String> {
        let (hue, sat) = match scene_name.to_lowercase().as_str() {
            "red" => (0, 254),
            "orange" => (6000, 254),
            "yellow" => (12000, 254),
            "green" => (25500, 254),
            "cyan" => (32000, 254),
            "blue" => (46920, 254),
            "purple" => (50000, 254),
            "pink" => (56000, 254),
            "white" => (0, 0),
            _ => return Err(format!("Unknown scene: {}", scene_name)),
        };

        let lights = self.get_lights().await?;

        for light in lights {
            self.set_light_color_with_transition(&light.id, hue, sat, transitiontime).await?;
        }

        Ok(())
    }

    /// Set all lights to a specific RGB color
    pub async fn set_all_lights_rgb(&self, r: u8, g: u8, b: u8) -> Result<(), String> {
        self.set_all_lights_rgb_with_transition(r, g, b, None).await
    }

    /// Set all lights to a specific RGB color with transition time (in deciseconds)
    pub async fn set_all_lights_rgb_with_transition(&self, r: u8, g: u8, b: u8, transitiontime: Option<u16>) -> Result<(), String> {
        let lights = self.get_lights().await?;

        for light in lights {
            self.set_light_rgb_with_transition(&light.id, r, g, b, transitiontime).await?;
        }

        Ok(())
    }

    /// Play an animated scene (cycle through color steps)
    /// Steps: Vec<(r, g, b, transition_time, duration_time)>
    pub async fn play_animated_scene(&self, steps: Vec<(u8, u8, u8, u16, u16)>) -> Result<(), String> {
        if steps.is_empty() {
            return Err("Scene has no color steps".to_string());
        }

        for (r, g, b, transition, duration) in steps {
            // Apply color with transition
            self.set_all_lights_rgb_with_transition(r, g, b, Some(transition)).await?;

            // Wait for transition + duration (convert deciseconds to milliseconds)
            let wait_time = (transition as u64 + duration as u64) * 100;
            tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
        }

        Ok(())
    }
}

/// Convert RGB to HSB for Hue lights
fn rgb_to_hsb(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Hue
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    // Saturation
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // Brightness
    let v = max;

    // Convert to Hue API values
    let hue = ((h / 360.0) * 65535.0) as u16;
    let sat = (s * 254.0) as u8;
    let bri = (v * 254.0) as u8;

    (hue, sat, bri)
}

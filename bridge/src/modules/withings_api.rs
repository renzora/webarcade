use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Withings API client
pub struct WithingsAPI {
    client: reqwest::Client,
    access_token: Arc<tokio::sync::RwLock<Option<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightMeasurement {
    pub date: i64,
    pub weight: f64,      // in kg
    pub fat_mass: Option<f64>,
    pub muscle_mass: Option<f64>,
    pub bone_mass: Option<f64>,
    pub hydration: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct WithingsResponse<T> {
    status: i32,
    body: T,
}

#[derive(Debug, Deserialize)]
struct MeasuresBody {
    measuregrps: Vec<MeasureGroup>,
}

#[derive(Debug, Deserialize)]
struct MeasureGroup {
    date: i64,
    measures: Vec<Measure>,
}

#[derive(Debug, Deserialize)]
struct Measure {
    #[serde(rename = "type")]
    measure_type: i32,
    value: i64,
    unit: i32,
}

impl WithingsAPI {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Set the access token
    pub async fn set_access_token(&self, token: String) {
        let mut access_token = self.access_token.write().await;
        *access_token = Some(token);
    }

    /// Get weight measurements
    /// startdate and enddate are Unix timestamps
    pub async fn get_weight_measurements(&self, startdate: Option<i64>, enddate: Option<i64>) -> Result<Vec<WeightMeasurement>> {
        let token = self.access_token.read().await;
        let token = token.as_ref().context("Not authenticated with Withings")?;

        let mut params = vec![
            ("action", "getmeas".to_string()),
            ("meastype", "1".to_string()), // Weight type
        ];

        if let Some(start) = startdate {
            params.push(("startdate", start.to_string()));
        }

        if let Some(end) = enddate {
            params.push(("enddate", end.to_string()));
        }

        let response = self.client
            .post("https://wbsapi.withings.net/measure")
            .bearer_auth(token)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Withings API request failed: {} - {}", status, text);
        }

        let data: WithingsResponse<MeasuresBody> = response.json().await?;

        if data.status != 0 {
            anyhow::bail!("Withings API error: status {}", data.status);
        }

        let mut measurements = Vec::new();

        for group in data.body.measuregrps {
            let mut weight: Option<f64> = None;
            let mut fat_mass: Option<f64> = None;
            let mut muscle_mass: Option<f64> = None;
            let mut bone_mass: Option<f64> = None;
            let mut hydration: Option<f64> = None;

            for measure in group.measures {
                let value = measure.value as f64 * 10f64.powi(measure.unit);

                match measure.measure_type {
                    1 => weight = Some(value),        // Weight (kg)
                    6 => fat_mass = Some(value),      // Fat mass (kg)
                    76 => muscle_mass = Some(value),  // Muscle mass (kg)
                    88 => bone_mass = Some(value),    // Bone mass (kg)
                    77 => hydration = Some(value),    // Hydration (kg)
                    _ => {}
                }
            }

            if let Some(w) = weight {
                measurements.push(WeightMeasurement {
                    date: group.date,
                    weight: w,
                    fat_mass,
                    muscle_mass,
                    bone_mass,
                    hydration,
                });
            }
        }

        Ok(measurements)
    }

    /// Get latest weight measurement
    pub async fn get_latest_weight(&self) -> Result<Option<WeightMeasurement>> {
        let measurements = self.get_weight_measurements(None, None).await?;
        Ok(measurements.into_iter().max_by_key(|m| m.date))
    }

    /// Get weight history for the last N days
    pub async fn get_weight_history(&self, days: i64) -> Result<Vec<WeightMeasurement>> {
        let now = chrono::Utc::now().timestamp();
        let start = now - (days * 86400);
        self.get_weight_measurements(Some(start), Some(now)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_withings_api_creation() {
        let api = WithingsAPI::new();
        assert!(api.access_token.read().await.is_none());
    }
}

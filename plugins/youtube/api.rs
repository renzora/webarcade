use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeChannel {
    pub id: String,
    pub title: String,
    pub description: String,
    pub custom_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub subscriber_count: Option<u64>,
    pub video_count: Option<u64>,
    pub view_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatistics {
    pub view_count: Option<u64>,
    pub subscriber_count: Option<u64>,
    pub video_count: Option<u64>,
    pub hidden_subscriber_count: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub views: Option<u64>,
    pub watch_time: Option<u64>,
    pub subscriber_change: Option<i64>,
    pub estimated_revenue: Option<f64>,
    pub average_view_duration: Option<u64>,
    pub likes: Option<u64>,
    pub dislikes: Option<u64>,
    pub comments: Option<u64>,
    pub shares: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub start_date: String,
    pub end_date: String,
    pub metrics: Vec<String>,
    pub dimensions: Option<Vec<String>>,
}

/// Get authenticated user's YouTube channels
pub async fn get_my_channels(access_token: &str) -> Result<Vec<YouTubeChannel>> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.googleapis.com/youtube/v3/channels")
        .query(&[
            ("part", "snippet,statistics,contentDetails"),
            ("mine", "true"),
        ])
        .bearer_auth(access_token)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to get channels: {}", error_text));
    }

    let data: Value = response.json().await?;
    let mut channels = Vec::new();

    if let Some(items) = data["items"].as_array() {
        for item in items {
            let channel = YouTubeChannel {
                id: item["id"].as_str().unwrap_or("").to_string(),
                title: item["snippet"]["title"].as_str().unwrap_or("").to_string(),
                description: item["snippet"]["description"].as_str().unwrap_or("").to_string(),
                custom_url: item["snippet"]["customUrl"].as_str().map(|s| s.to_string()),
                thumbnail_url: item["snippet"]["thumbnails"]["high"]["url"]
                    .as_str()
                    .map(|s| s.to_string()),
                subscriber_count: item["statistics"]["subscriberCount"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                video_count: item["statistics"]["videoCount"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                view_count: item["statistics"]["viewCount"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
            };
            channels.push(channel);
        }
    }

    Ok(channels)
}

/// Get channel by ID
pub async fn get_channel_by_id(access_token: &str, channel_id: &str) -> Result<YouTubeChannel> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.googleapis.com/youtube/v3/channels")
        .query(&[
            ("part", "snippet,statistics,contentDetails"),
            ("id", channel_id),
        ])
        .bearer_auth(access_token)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to get channel: {}", error_text));
    }

    let data: Value = response.json().await?;

    if let Some(items) = data["items"].as_array() {
        if let Some(item) = items.first() {
            return Ok(YouTubeChannel {
                id: item["id"].as_str().unwrap_or("").to_string(),
                title: item["snippet"]["title"].as_str().unwrap_or("").to_string(),
                description: item["snippet"]["description"].as_str().unwrap_or("").to_string(),
                custom_url: item["snippet"]["customUrl"].as_str().map(|s| s.to_string()),
                thumbnail_url: item["snippet"]["thumbnails"]["high"]["url"]
                    .as_str()
                    .map(|s| s.to_string()),
                subscriber_count: item["statistics"]["subscriberCount"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                video_count: item["statistics"]["videoCount"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                view_count: item["statistics"]["viewCount"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
            });
        }
    }

    Err(anyhow!("Channel not found"))
}

/// Get YouTube Analytics report
pub async fn get_analytics_report(
    access_token: &str,
    channel_id: &str,
    query: &AnalyticsQuery,
) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut params = vec![
        ("ids", format!("channel=={}", channel_id)),
        ("startDate", query.start_date.clone()),
        ("endDate", query.end_date.clone()),
        ("metrics", query.metrics.join(",")),
    ];

    if let Some(dimensions) = &query.dimensions {
        params.push(("dimensions", dimensions.join(",")));
    }

    let response = client
        .get("https://youtubeanalytics.googleapis.com/v2/reports")
        .query(&params)
        .bearer_auth(access_token)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to get analytics: {}", error_text));
    }

    let data: Value = response.json().await?;
    Ok(data)
}

/// Get channel analytics summary for a date range
pub async fn get_channel_analytics(
    access_token: &str,
    channel_id: &str,
    start_date: &str,
    end_date: &str,
) -> Result<AnalyticsMetrics> {
    let metrics = vec![
        "views".to_string(),
        "estimatedMinutesWatched".to_string(),
        "subscribersGained".to_string(),
        "subscribersLost".to_string(),
        "averageViewDuration".to_string(),
        "likes".to_string(),
        "comments".to_string(),
        "shares".to_string(),
    ];

    let query = AnalyticsQuery {
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        metrics,
        dimensions: None,
    };

    let data = get_analytics_report(access_token, channel_id, &query).await?;

    // Parse the response
    let mut analytics = AnalyticsMetrics {
        views: None,
        watch_time: None,
        subscriber_change: None,
        estimated_revenue: None,
        average_view_duration: None,
        likes: None,
        dislikes: None,
        comments: None,
        shares: None,
    };

    if let Some(rows) = data["rows"].as_array() {
        if let Some(row) = rows.first() {
            if let Some(values) = row.as_array() {
                if values.len() >= 8 {
                    analytics.views = values[0].as_u64();
                    analytics.watch_time = values[1].as_u64();
                    let gained = values[2].as_i64().unwrap_or(0);
                    let lost = values[3].as_i64().unwrap_or(0);
                    analytics.subscriber_change = Some(gained - lost);
                    analytics.average_view_duration = values[4].as_u64();
                    analytics.likes = values[5].as_u64();
                    analytics.comments = values[6].as_u64();
                    analytics.shares = values[7].as_u64();
                }
            }
        }
    }

    Ok(analytics)
}

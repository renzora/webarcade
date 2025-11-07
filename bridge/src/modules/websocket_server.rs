use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use log::{info, warn, error, debug};
use tokio::sync::broadcast;
use crate::file_watcher::get_file_change_receiver;
// use super::twitch::{TwitchEvent, TwitchManager}; // Commented out - using plugin system now

pub type WsSink = SplitSink<WebSocketStream<TcpStream>, Message>;

// Global managers for WebSocket access - DISABLED (using plugin system)
// static TWITCH_MANAGER: tokio::sync::OnceCell<Option<Arc<TwitchManager>>> = tokio::sync::OnceCell::const_new();

// Timer broadcast channel
static TIMER_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_timer_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    TIMER_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_timer_state(state: serde_json::Value) {
    let sender = get_timer_broadcast_sender();
    let _ = sender.send(state);
}

// Goals broadcast channel
static GOALS_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_goals_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    GOALS_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_goals_update(goals: serde_json::Value) {
    let sender = get_goals_broadcast_sender();
    let message = serde_json::json!({
        "type": "goals_update",
        "goals": goals
    });
    let _ = sender.send(message);
}

// Status config broadcast channel (for ticker speed and stream start date)
static STATUS_CONFIG_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_status_config_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    STATUS_CONFIG_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_status_config_update(config: serde_json::Value) {
    let sender = get_status_config_broadcast_sender();
    let message = serde_json::json!({
        "type": "status_config_update",
        "config": config
    });
    let _ = sender.send(message);
}

// Ticker messages broadcast channel
static TICKER_MESSAGES_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_ticker_messages_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    TICKER_MESSAGES_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_ticker_messages_update() {
    let sender = get_ticker_messages_broadcast_sender();
    let message = serde_json::json!({
        "type": "ticker_messages_update"
    });
    let _ = sender.send(message);
}

// Breaking news broadcast channel
static BREAKING_NEWS_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_breaking_news_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    BREAKING_NEWS_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_breaking_news_update(breaking_news: serde_json::Value) {
    let sender = get_breaking_news_broadcast_sender();
    let message = serde_json::json!({
        "type": "breaking_news_update",
        "breaking_news": breaking_news
    });
    let _ = sender.send(message);
}

// Ticker segments broadcast channel
static TICKER_SEGMENTS_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_ticker_segments_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    TICKER_SEGMENTS_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_ticker_segments_update() {
    let sender = get_ticker_segments_broadcast_sender();
    let message = serde_json::json!({
        "type": "ticker_segments_update"
    });
    let _ = sender.send(message);
}

// Segment duration broadcast channel
static SEGMENT_DURATION_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_segment_duration_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    SEGMENT_DURATION_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_segment_duration_update(duration: i64) {
    let sender = get_segment_duration_broadcast_sender();
    let message = serde_json::json!({
        "type": "segment_duration_update",
        "duration": duration
    });
    let _ = sender.send(message);
}

// Mood ticker broadcast channel
static MOOD_TICKER_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_mood_ticker_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    MOOD_TICKER_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

// Commented out - depends on old database.rs
// pub fn broadcast_mood_ticker_update(data: crate::commands::database::MoodTickerData) {
//     let sender = get_mood_ticker_broadcast_sender();
//     let message = serde_json::json!({
//         "type": "mood_ticker_update",
//         "data": data
//     });
//     let _ = sender.send(message);
// }

// Layout broadcast channel
static LAYOUT_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_layout_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    LAYOUT_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_layout_update(layout_name: String, layout_data: serde_json::Value) {
    let sender = get_layout_broadcast_sender();
    let message = serde_json::json!({
        "type": "layout_update",
        "layout_name": layout_name,
        "layout": layout_data
    });
    let _ = sender.send(message);
}

// Community Tasks broadcast channel
static COMMUNITY_TASKS_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_community_tasks_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    COMMUNITY_TASKS_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_community_tasks_toggle(enabled: bool) {
    let sender = get_community_tasks_broadcast_sender();
    let message = serde_json::json!({
        "type": "community_tasks_toggle",
        "enabled": enabled
    });
    let _ = sender.send(message);
}

// Chat Highlight broadcast channel
static CHAT_HIGHLIGHT_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_chat_highlight_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    CHAT_HIGHLIGHT_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_chat_highlight(highlight_data: serde_json::Value) {
    let sender = get_chat_highlight_broadcast_sender();
    let _ = sender.send(highlight_data);
}

// Pack opening broadcast channel
static PACK_OPENING_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_pack_opening_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    PACK_OPENING_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_pack_opening(pack_data: serde_json::Value) {
    let sender = get_pack_opening_broadcast_sender();
    let _ = sender.send(pack_data);
}

// Roulette broadcast channel
static ROULETTE_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_roulette_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    ROULETTE_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_roulette_update(roulette_data: serde_json::Value) {
    let sender = get_roulette_broadcast_sender();
    let _ = sender.send(roulette_data);
}

// Auction broadcast channel
static AUCTION_BROADCAST: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

pub fn get_auction_broadcast_sender() -> broadcast::Sender<serde_json::Value> {
    AUCTION_BROADCAST.get_or_init(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    }).clone()
}

pub fn broadcast_auction_update(auction_data: serde_json::Value) {
    let sender = get_auction_broadcast_sender();
    let _ = sender.send(auction_data);
}

// Commented out - using plugin system now
// pub async fn set_twitch_manager(manager: Option<Arc<TwitchManager>>) {
//     let _ = TWITCH_MANAGER.set(manager);
// }

// pub async fn get_twitch_manager() -> Option<Arc<TwitchManager>> {
//     TWITCH_MANAGER.get().and_then(|m| m.clone())
// }

pub async fn start_websocket_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(&addr).await?;
    info!("🔌 WebSocket server listening on: ws://{}", addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        info!("📡 New WebSocket connection from: {}", client_addr);
        tokio::spawn(handle_websocket_connection(stream, client_addr));
    }

    Ok(())
}

async fn handle_websocket_connection(stream: TcpStream, client_addr: SocketAddr) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed for {}: {}", client_addr, e);
            return;
        }
    };

    info!("WebSocket connection established with {}", client_addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send initial connection message with channel info
    let mut welcome_msg = serde_json::json!({
        "type": "connected",
        "message": "WebSocket connection established"
    });

    // Get channel from database
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.query_row(
                "SELECT username FROM twitch_auth LIMIT 1",
                [],
                |row| row.get::<_, String>(0)
            ) {
                Ok(username) => {
                    welcome_msg["channel"] = serde_json::json!(username);
                    log::info!("[WebSocket] Sending channel '{}' in welcome message", username);
                }
                Err(e) => {
                    log::error!("[WebSocket] Failed to query username from twitch_auth: {}", e);
                }
            }
        }
        Err(e) => {
            log::error!("[WebSocket] Failed to open database at {:?}: {}", db_path, e);
        }
    }

    if let Ok(welcome_text) = serde_json::to_string(&welcome_msg) {
        if let Err(e) = ws_sender.send(Message::Text(welcome_text)).await {
            error!("Failed to send welcome message to {}: {}", client_addr, e);
            return;
        }
    }

    // Get file change receiver
    let mut file_change_receiver = match get_file_change_receiver() {
        Some(receiver) => receiver,
        None => {
            error!("No file change receiver available for {}", client_addr);
            return;
        }
    };

    // Get Twitch event receiver if available - DISABLED (using plugin system)
    // let mut twitch_receiver_opt: Option<broadcast::Receiver<TwitchEvent>> = None;
    // if let Some(twitch_manager) = get_twitch_manager().await {
    //     let receiver = twitch_manager.get_event_sender().subscribe();
    //     twitch_receiver_opt = Some(receiver);
    //     info!("📡 WebSocket client {} subscribed to Twitch events", client_addr);
    // }

    // Get timer event receiver
    let mut timer_receiver = get_timer_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to timer events", client_addr);

    // Get goals event receiver
    let mut goals_receiver = get_goals_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to goals events", client_addr);

    // Get status config event receiver
    let mut status_config_receiver = get_status_config_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to status config events", client_addr);

    // Get ticker messages event receiver
    let mut ticker_messages_receiver = get_ticker_messages_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to ticker messages events", client_addr);

    // Get breaking news event receiver
    let mut breaking_news_receiver = get_breaking_news_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to breaking news events", client_addr);

    // Get ticker segments event receiver
    let mut ticker_segments_receiver = get_ticker_segments_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to ticker segments events", client_addr);

    // Get segment duration event receiver
    let mut segment_duration_receiver = get_segment_duration_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to segment duration events", client_addr);

    // Get mood ticker event receiver
    let mut mood_ticker_receiver = get_mood_ticker_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to mood ticker events", client_addr);

    // Get layout event receiver
    let mut layout_receiver = get_layout_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to layout events", client_addr);

    // Get community tasks event receiver
    let mut community_tasks_receiver = get_community_tasks_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to community tasks events", client_addr);

    // Get chat highlight event receiver
    let mut chat_highlight_receiver = get_chat_highlight_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to chat highlight events", client_addr);

    // Get pack opening event receiver
    let mut pack_opening_receiver = get_pack_opening_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to pack opening events", client_addr);

    // Get roulette event receiver
    let mut roulette_receiver = get_roulette_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to roulette events", client_addr);

    // Get auction event receiver
    let mut auction_receiver = get_auction_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to auction events", client_addr);

    info!("📡 WebSocket client {} subscribed to file changes", client_addr);

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages from client
            ws_msg = ws_receiver.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        debug!("📡 Received message from {}: {}", client_addr, text);

                        // Try to parse and handle layout updates from clients
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            if json.get("type").and_then(|t| t.as_str()) == Some("layout_update") {
                                // Client is sending a layout update - broadcast it to all other clients
                                if let (Some(layout_name), Some(layout_data)) = (
                                    json.get("layout_name").and_then(|n| n.as_str()),
                                    json.get("layout")
                                ) {
                                    info!("📡 Received layout update from client for: {}", layout_name);
                                    broadcast_layout_update(layout_name.to_string(), layout_data.clone());
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("📡 WebSocket connection closed by client {}", client_addr);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error from {}: {}", client_addr, e);
                        break;
                    }
                    None => {
                        info!("📡 WebSocket stream ended for {}", client_addr);
                        break;
                    }
                    _ => {}
                }
            }

            // Handle file change events
            file_change = file_change_receiver.recv() => {
                match file_change {
                    Ok(change_message) => {
                        debug!("📡 Broadcasting file change to {}: {}", client_addr, change_message);

                        let ws_message = serde_json::json!({
                            "type": "file_change",
                            "message": change_message
                        });

                        if let Ok(message_text) = serde_json::to_string(&ws_message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send file change to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 File change broadcaster closed, ending WebSocket connection for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle Twitch events (if subscribed) - DISABLED (using plugin system WebSocket bridge)
            // twitch_event = async {
            //     if let Some(ref mut receiver) = twitch_receiver_opt {
            //         Some(receiver.recv().await)
            //     } else {
            //         std::future::pending::<Option<Result<TwitchEvent, tokio::sync::broadcast::error::RecvError>>>().await
            //     }
            // }, if twitch_receiver_opt.is_some() => {
            //     let twitch_event = twitch_event.unwrap();
            //     match twitch_event {
            //         Ok(event) => {
            //             debug!("📡 Broadcasting Twitch event to {}", client_addr);

            //             let ws_message = serde_json::json!({
            //                 "type": "twitch_event",
            //                 "event": event
            //             });
            //
            //             if let Ok(message_text) = serde_json::to_string(&ws_message) {
            //                 if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
            //                     error!("Failed to send Twitch event to {}: {}", client_addr, e);
            //                     break;
            //                 }
            //             }
            //         }
            //         Err(broadcast::error::RecvError::Lagged(count)) => {
            //             warn!("📡 WebSocket client {} lagged {} Twitch messages", client_addr, count);
            //         }
            //         Err(broadcast::error::RecvError::Closed) => {
            //             info!("📡 Twitch event broadcaster closed for {}", client_addr);
            //             twitch_receiver_opt = None;
            //         }
            //     }
            // }

            // Handle timer events
            timer_event = timer_receiver.recv() => {
                match timer_event {
                    Ok(state) => {
                        debug!("📡 Broadcasting timer state to {}", client_addr);

                        let ws_message = serde_json::json!({
                            "type": "timer_state",
                            "name": state.get("name"),
                            "isRunning": state.get("isRunning"),
                            "isPaused": state.get("isPaused"),
                            "timeRemaining": state.get("timeRemaining"),
                            "isPomodoro": state.get("isPomodoro"),
                            "currentPhase": state.get("currentPhase"),
                            "pomodoroCount": state.get("pomodoroCount")
                        });

                        if let Ok(message_text) = serde_json::to_string(&ws_message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send timer state to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} timer messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Timer broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle goals events
            goals_event = goals_receiver.recv() => {
                match goals_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting goals update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send goals update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} goals messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Goals broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle status config events (ticker speed, stream start date)
            status_config_event = status_config_receiver.recv() => {
                match status_config_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting status config update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send status config update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} status config messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Status config broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle ticker messages events
            ticker_messages_event = ticker_messages_receiver.recv() => {
                match ticker_messages_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting ticker messages update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send ticker messages update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} ticker messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Ticker messages broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle breaking news events
            breaking_news_event = breaking_news_receiver.recv() => {
                match breaking_news_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting breaking news update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send breaking news update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} breaking news messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Breaking news broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle ticker segments events
            ticker_segments_event = ticker_segments_receiver.recv() => {
                match ticker_segments_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting ticker segments update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send ticker segments update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} ticker segments messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Ticker segments broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle segment duration events
            segment_duration_event = segment_duration_receiver.recv() => {
                match segment_duration_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting segment duration update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send segment duration update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} segment duration messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Segment duration broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle mood ticker events
            mood_ticker_event = mood_ticker_receiver.recv() => {
                match mood_ticker_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting mood ticker update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send mood ticker update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} mood ticker messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Mood ticker broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle layout events
            layout_event = layout_receiver.recv() => {
                match layout_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting layout update to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send layout update to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} layout messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Layout broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle community tasks events
            community_tasks_event = community_tasks_receiver.recv() => {
                match community_tasks_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting community tasks toggle to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send community tasks toggle to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} community tasks messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Community tasks broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle chat highlight events
            chat_highlight_event = chat_highlight_receiver.recv() => {
                match chat_highlight_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting chat highlight to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send chat highlight to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} chat highlight messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Chat highlight broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle pack opening events
            pack_opening_event = pack_opening_receiver.recv() => {
                match pack_opening_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting pack opening to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send pack opening to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} pack opening messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Pack opening broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle roulette events
            roulette_event = roulette_receiver.recv() => {
                match roulette_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting roulette event to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send roulette event to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} roulette messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Roulette broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }

            // Handle auction events
            auction_event = auction_receiver.recv() => {
                match auction_event {
                    Ok(message) => {
                        debug!("📡 Broadcasting auction event to {}", client_addr);

                        if let Ok(message_text) = serde_json::to_string(&message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send auction event to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} auction messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Auction broadcaster closed for {}", client_addr);
                        break;
                    }
                }
            }
        }
    }

    info!("📡 WebSocket connection with {} ended", client_addr);
}
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use log::{info, warn, error, debug};
use tokio::sync::broadcast;
use crate::file_watcher::get_file_change_receiver;
use super::twitch::{TwitchEvent, TwitchManager};

pub type WsSink = SplitSink<WebSocketStream<TcpStream>, Message>;

// Global managers for WebSocket access
static TWITCH_MANAGER: tokio::sync::OnceCell<Option<Arc<TwitchManager>>> = tokio::sync::OnceCell::const_new();

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

pub async fn set_twitch_manager(manager: Option<Arc<TwitchManager>>) {
    let _ = TWITCH_MANAGER.set(manager);
}

pub async fn get_twitch_manager() -> Option<Arc<TwitchManager>> {
    TWITCH_MANAGER.get().and_then(|m| m.clone())
}

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

    // Try to get channel from Twitch manager config
    if let Some(twitch_manager) = get_twitch_manager().await {
        let config_manager = twitch_manager.get_config_manager();
        if let Ok(config) = config_manager.load() {
            if let Some(first_channel) = config.channels.first() {
                welcome_msg["channel"] = serde_json::json!(first_channel);
            }
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

    // Get Twitch event receiver if available
    let mut twitch_receiver_opt: Option<broadcast::Receiver<TwitchEvent>> = None;
    if let Some(twitch_manager) = get_twitch_manager().await {
        let receiver = twitch_manager.get_event_sender().subscribe();
        twitch_receiver_opt = Some(receiver);
        info!("📡 WebSocket client {} subscribed to Twitch events", client_addr);
    }

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

    // Get layout event receiver
    let mut layout_receiver = get_layout_broadcast_sender().subscribe();
    info!("📡 WebSocket client {} subscribed to layout events", client_addr);

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

            // Handle Twitch events (if subscribed)
            twitch_event = async {
                if let Some(ref mut receiver) = twitch_receiver_opt {
                    Some(receiver.recv().await)
                } else {
                    std::future::pending::<Option<Result<TwitchEvent, tokio::sync::broadcast::error::RecvError>>>().await
                }
            }, if twitch_receiver_opt.is_some() => {
                let twitch_event = twitch_event.unwrap();
                match twitch_event {
                    Ok(event) => {
                        debug!("📡 Broadcasting Twitch event to {}", client_addr);

                        let ws_message = serde_json::json!({
                            "type": "twitch_event",
                            "event": event
                        });

                        if let Ok(message_text) = serde_json::to_string(&ws_message) {
                            if let Err(e) = ws_sender.send(Message::Text(message_text)).await {
                                error!("Failed to send Twitch event to {}: {}", client_addr, e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("📡 WebSocket client {} lagged {} Twitch messages", client_addr, count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("📡 Twitch event broadcaster closed for {}", client_addr);
                        twitch_receiver_opt = None;
                    }
                }
            }

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
        }
    }

    info!("📡 WebSocket connection with {} ended", client_addr);
}
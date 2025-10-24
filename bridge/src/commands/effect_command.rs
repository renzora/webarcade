use crate::modules::twitch::{CommandSystem, Command, PermissionLevel, TwitchIRCManager};
use crate::modules::twitch::twitch_irc_client::{TwitchEvent, EffectTriggerEvent};
use std::sync::Arc;

/// Register effect command
pub async fn register(command_system: &CommandSystem) {
    let effect_command = Command {
        name: "effect".to_string(),
        aliases: vec![],
        description: "Trigger a crazy 3D effect overlay".to_string(),
        usage: "!effect".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 30,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let username = ctx.message.username.clone();

            Ok(Some(tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    handle_effect_command(&channel, &username, irc).await
                })
            })))
        }),
    };

    command_system.register_command(effect_command).await;
}

/// Handle the !effect command
async fn handle_effect_command(
    channel: &str,
    username: &str,
    irc: Arc<TwitchIRCManager>,
) -> String {
    // Broadcast the effect trigger event via WebSocket
    let effect_event = TwitchEvent::EffectTrigger(EffectTriggerEvent {
        channel: channel.to_string(),
        triggered_by: username.to_string(),
    });

    let event_sender = irc.get_event_sender();
    if let Err(e) = event_sender.send(effect_event) {
        log::error!("Failed to broadcast effect trigger event: {}", e);
    }

    format!("💥 @{} triggered the crazy effect!", username)
}

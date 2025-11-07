// All plugins are registered here

// Tier 0: Core plugins
pub mod database;

// Tier 1: Foundational plugins
pub mod currency;
pub mod notes;
pub mod goals;
pub mod todos;
pub mod counters;

// Tier 2: Game plugins
pub mod auction;
pub mod roulette;
pub mod levels;
pub mod wheel;
pub mod packs;

// Tier 3: Utility plugins
pub mod files;
pub mod system;
pub mod status;
pub mod ticker;
pub mod text_commands;
pub mod user_profiles;
pub mod tts;
pub mod confessions;
pub mod household;
pub mod song_requests;
pub mod mood_tracker;
pub mod timer;
pub mod overlay_manager;
pub mod overlays; // Only serves layout HTML generation
pub mod layouts;

// Tier 4: Integration plugins
pub mod twitch;
pub mod alerts;
pub mod hue;
pub mod withings;

// Tier 5: Analytics and engagement plugins
pub mod watchtime;
pub mod followers;

// Plugin stubs (not yet implemented)
// pub mod fun_commands;
pub mod discord;
pub mod alexa;
// pub mod obs;

use crate::core::plugin_manager::PluginManager;

/// Register all plugins with the plugin manager
/// Plugins are loaded in dependency order automatically
pub fn register_all_plugins(manager: &mut PluginManager) {
    log::info!("📦 Registering plugins...");

    // Tier 0: Core plugins
    manager.register(database::DatabasePlugin);

    // Tier 1: Foundational plugins (no dependencies)
    manager.register(currency::CurrencyPlugin);
    manager.register(notes::NotesPlugin);
    manager.register(goals::GoalsPlugin);
    manager.register(todos::TodosPlugin);
    manager.register(counters::CountersPlugin);

    // Tier 2: Game plugins (depend on currency)
    manager.register(auction::AuctionPlugin);
    manager.register(roulette::RoulettePlugin);
    manager.register(levels::LevelsPlugin);
    manager.register(wheel::WheelPlugin);
    manager.register(packs::PacksPlugin);

    // Tier 3: Utility plugins
    manager.register(files::FilesPlugin);
    manager.register(system::SystemPlugin);
    manager.register(status::StatusPlugin);
    manager.register(ticker::TickerPlugin);
    manager.register(text_commands::TextCommandsPlugin);
    manager.register(user_profiles::UserProfilesPlugin);
    manager.register(tts::TtsPlugin);
    manager.register(confessions::ConfessionsPlugin);
    manager.register(household::HouseholdPlugin);
    manager.register(song_requests::SongRequestsPlugin);
    manager.register(mood_tracker::MoodTrackerPlugin);
    manager.register(timer::TimerPlugin);
    manager.register(overlay_manager::OverlayManagerPlugin);
    manager.register(overlays::OverlaysPlugin); // Only serves layout HTML
    manager.register(layouts::LayoutsPlugin);

    // Tier 4: Integration plugins
    manager.register(twitch::TwitchPlugin);
    manager.register(alerts::AlertsPlugin);
    manager.register(hue::HuePlugin);
    manager.register(withings::WithingsPlugin);

    // Tier 5: Analytics and engagement plugins
    manager.register(watchtime::WatchtimePlugin);
    manager.register(followers::FollowersPlugin);

    // TODO: Uncomment as you implement remaining plugins:
    // manager.register(fun_commands::FunCommandsPlugin);
    manager.register(discord::DiscordPlugin);
    manager.register(alexa::AlexaPlugin);
    // manager.register(obs::ObsPlugin);

    log::info!("✅ Plugin registration complete (33 plugins)");
}

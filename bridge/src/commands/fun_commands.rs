use crate::modules::twitch::{CommandSystem, Command, PermissionLevel};
use std::sync::Arc;
use rand::Rng;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
struct JokeApiResponse {
    joke: Option<String>,
    setup: Option<String>,
    delivery: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DadJokeResponse {
    joke: String,
}

#[derive(Deserialize, Debug)]
struct YoMommaResponse {
    joke: String,
}

#[derive(Deserialize, Debug)]
struct QuotableResponse {
    content: String,
    author: String,
}

#[derive(Deserialize, Debug)]
struct EightBallResponse {
    reading: String,
}

/// Fetch a random joke from JokeAPI
async fn fetch_joke() -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get("https://v2.jokeapi.dev/joke/Any?safe-mode")
        .await?
        .json::<JokeApiResponse>()
        .await?;

    if let Some(joke) = response.joke {
        Ok(joke)
    } else if let (Some(setup), Some(delivery)) = (response.setup, response.delivery) {
        Ok(format!("{} - {}", setup, delivery))
    } else {
        Err("Failed to parse joke".into())
    }
}

/// Fetch a random dad joke
async fn fetch_dad_joke() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://icanhazdadjoke.com/")
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<DadJokeResponse>()
        .await?;

    Ok(response.joke)
}

/// Fetch a random yo momma joke using official joke API
async fn fetch_yo_momma_joke() -> Result<String, Box<dyn std::error::Error>> {
    // Using official-joke-api which is more reliable
    let response = reqwest::get("https://official-joke-api.appspot.com/jokes/general/random")
        .await?
        .text()
        .await?;

    // Parse as array since the API returns an array with one joke
    let jokes: Vec<serde_json::Value> = serde_json::from_str(&response)?;

    if let Some(joke) = jokes.first() {
        if let (Some(setup), Some(punchline)) = (joke.get("setup"), joke.get("punchline")) {
            return Ok(format!("{} {}",
                setup.as_str().unwrap_or(""),
                punchline.as_str().unwrap_or("")
            ));
        }
    }

    Err("Failed to parse joke".into())
}

/// Fetch a random quote using ZenQuotes
async fn fetch_quote() -> Result<String, Box<dyn std::error::Error>> {
    // Using ZenQuotes API which doesn't require authentication
    let response = reqwest::get("https://zenquotes.io/api/random")
        .await?
        .text()
        .await?;

    // Parse the response as an array
    let quotes: Vec<serde_json::Value> = serde_json::from_str(&response)?;

    if let Some(quote_obj) = quotes.first() {
        let quote = quote_obj.get("q").and_then(|v| v.as_str()).unwrap_or("");
        let author = quote_obj.get("a").and_then(|v| v.as_str()).unwrap_or("Unknown");
        return Ok(format!("\"{}\" - {}", quote, author));
    }

    Err("Failed to parse quote".into())
}

/// Fetch a magic 8-ball response
async fn fetch_8ball() -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get("https://www.eightballapi.com/api")
        .await?
        .json::<EightBallResponse>()
        .await?;

    Ok(response.reading)
}

/// Generate a random roast
fn generate_roast() -> String {
    let roasts = vec![
        "I'd agree with you, but then we'd both be wrong.",
        "You're not stupid; you just have bad luck thinking.",
        "If I wanted to kill myself, I'd climb your ego and jump to your IQ.",
        "I'm jealous of people who don't know you.",
        "You bring everyone so much joy... when you leave the room.",
        "I'd explain it to you, but I left my English-to-Dingbat dictionary at home.",
        "You're the reason gene pools need lifeguards.",
        "Somewhere out there is a tree tirelessly producing oxygen for you. You owe it an apology.",
        "I'd roast you, but my mom said I'm not allowed to burn trash.",
        "You're as bright as a black hole and twice as dense.",
        "I've seen people like you before, but I had to pay admission.",
        "Your face makes onions cry.",
        "If laughter is the best medicine, your face must be curing the world.",
        "You're proof that evolution can go in reverse.",
        "I'm not saying you're dumb, but you have the intellectual range of a potato.",
        "You have the perfect face for radio.",
        "I'd call you a tool, but that would imply you're useful.",
        "Your secrets are safe with me. I wasn't even listening.",
        "I hope your day is as pleasant as you are.",
        "You're like a software update. Whenever I see you, I think 'not now'.",
    ];

    let mut rng = rand::thread_rng();
    roasts[rng.gen_range(0..roasts.len())].to_string()
}

pub async fn register(command_system: &CommandSystem) {
    // Register !joke command
    let joke_command = Command {
        name: "joke".to_string(),
        aliases: vec![],
        description: "Get a random joke".to_string(),
        usage: "!joke".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(|ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();

            tokio::spawn(async move {
                let message = match fetch_joke().await {
                    Ok(joke) => format!("😂 {}", joke),
                    Err(e) => {
                        log::error!("Failed to fetch joke: {}", e);
                        "Sorry, couldn't fetch a joke right now!".to_string()
                    }
                };
                let _ = irc.send_message(&channel, &message).await;
            });

            Ok(None)
        }),
    };

    // Register !dadjoke command
    let dadjoke_command = Command {
        name: "dadjoke".to_string(),
        aliases: vec![],
        description: "Get a random dad joke".to_string(),
        usage: "!dadjoke".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(|ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();

            tokio::spawn(async move {
                let message = match fetch_dad_joke().await {
                    Ok(joke) => format!("👨 {}", joke),
                    Err(e) => {
                        log::error!("Failed to fetch dad joke: {}", e);
                        "Sorry, couldn't fetch a dad joke right now!".to_string()
                    }
                };
                let _ = irc.send_message(&channel, &message).await;
            });

            Ok(None)
        }),
    };

    // Register !8ball command
    let eightball_command = Command {
        name: "8ball".to_string(),
        aliases: vec!["eightball".to_string()],
        description: "Ask the magic 8-ball a question".to_string(),
        usage: "!8ball <question>".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 3,
        enabled: true,
        handler: Arc::new(|ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let username = ctx.message.username.clone();

            tokio::spawn(async move {
                let message = match fetch_8ball().await {
                    Ok(reading) => format!("🔮 @{}: {}", username, reading),
                    Err(e) => {
                        log::error!("Failed to fetch 8-ball response: {}", e);
                        "Sorry, the magic 8-ball is unavailable!".to_string()
                    }
                };
                let _ = irc.send_message(&channel, &message).await;
            });

            Ok(None)
        }),
    };

    // Register !quote command
    let quote_command = Command {
        name: "quote".to_string(),
        aliases: vec![],
        description: "Get a random inspirational quote".to_string(),
        usage: "!quote".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(|ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();

            tokio::spawn(async move {
                let message = match fetch_quote().await {
                    Ok(quote) => format!("💭 {}", quote),
                    Err(e) => {
                        log::error!("Failed to fetch quote: {}", e);
                        "Sorry, couldn't fetch a quote right now!".to_string()
                    }
                };
                let _ = irc.send_message(&channel, &message).await;
            });

            Ok(None)
        }),
    };

    // Register !roast command
    let roast_command = Command {
        name: "roast".to_string(),
        aliases: vec![],
        description: "Get roasted!".to_string(),
        usage: "!roast [@username]".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(|ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let username = ctx.message.username.clone();
            let args = ctx.args.clone();

            tokio::spawn(async move {
                let roast = generate_roast();

                // Check if there's a target mentioned
                let target = if !args.is_empty() {
                    args[0].trim_start_matches('@').to_string()
                } else {
                    username.clone()
                };

                let _ = irc.send_message(&channel, &format!("🔥 @{}: {}", target, roast)).await;
            });

            Ok(None)
        }),
    };

    // Register !yomomma command
    let yomomma_command = Command {
        name: "yomomma".to_string(),
        aliases: vec!["ymj".to_string()],
        description: "Get a random joke".to_string(),
        usage: "!yomomma".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(|ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();

            tokio::spawn(async move {
                let message = match fetch_yo_momma_joke().await {
                    Ok(joke) => format!("🤣 {}", joke),
                    Err(e) => {
                        log::error!("Failed to fetch yo momma joke: {}", e);
                        "Sorry, couldn't fetch a yo momma joke right now!".to_string()
                    }
                };
                let _ = irc.send_message(&channel, &message).await;
            });

            Ok(None)
        }),
    };

    command_system.register_command(joke_command).await;
    command_system.register_command(dadjoke_command).await;
    command_system.register_command(eightball_command).await;
    command_system.register_command(quote_command).await;
    command_system.register_command(roast_command).await;
    command_system.register_command(yomomma_command).await;

    log::info!("✅ Registered fun commands (joke, dadjoke, 8ball, quote, roast, yomomma)");
}

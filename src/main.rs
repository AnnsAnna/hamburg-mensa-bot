use serenity::model::Timestamp;
use std::collections::HashMap;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
extern crate dotenv;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tokio::task::spawn_blocking;

mod meals;

use dotenv::dotenv;
struct Handler {
    meals: Option<Vec<meals::Meal>>,
    channel_id: u64,
}

const BASE_URL: &str =
    "https://raw.githubusercontent.com/HAWHHCalendarBot/mensa-data/main/Mensa%20Berliner%20Tor/";
const USER_AGENT: &str = "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let channel = ctx.http.get_channel(self.channel_id).await.unwrap().id();
        
        // Clear the channel
        let messages = channel.messages(&ctx.http, |retriever| retriever.limit(100)).await.unwrap();
        channel.delete_messages(&ctx.http, messages).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file.
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id = env::var("CHANNEL_ID").expect("Expected a channel id in the environment");

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap();

    let now = chrono::Local::now();
    let request = client
        .get(format!("{}{}.json", BASE_URL, now.format("%Y/%m/%d")))
        .send()
        .await
        .unwrap();

    println!("Status: {}", request.status());

    let meals = if request.status() == 404 {
        None
    } else {
        Some(request.json::<Vec<meals::Meal>>().await.unwrap())
    };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            meals: meals,
            channel_id: channel_id.parse::<u64>().unwrap(),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

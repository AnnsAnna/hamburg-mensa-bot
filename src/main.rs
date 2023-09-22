use mensa_crawler::meal::{Meal, Meta};
use mensa_crawler::{http, meal, parse, write_meals};
use serenity::model::Timestamp;
use std::collections::HashMap;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
extern crate dotenv;
use chrono::{DateTime, Utc};

use dotenv::dotenv;
struct Handler {
    meals: Vec<Meal>,
}

const CANTEENS: [&str; 2] = ["Mensa Berliner Tor", "Mensa Bergedorf"];
const URL_THIS_WEEK: &str = "https://www.stwhh.de/speiseplan/?t=next_day";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.http.send_message(1140056759118082109, |m| {
            m.content("Hello, World!")
                .embed(|e| {
                    e.title("This is a title")
                        .description("This is a description")
                        .image("attachment://ferris_eyes.png")
                        .fields(vec![
                            ("This is the first field", "This is a field body", true),
                            ("This is the second field", "Both fields are inline", true),
                        ])
                        .field("This is the third field", "This is not an inline field", false)
                        .footer(|f| f.text("This is a footer"))
                        // Add a timestamp for the current time
                        // This also accepts a rfc3339 Timestamp
                        .timestamp(Timestamp::now())
                })
        }).await.unwrap();
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file.
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    println!("this week...");
    let html = http::get_text(URL_THIS_WEEK).unwrap();
    let meals = parse::parse(&html);
    let mut canteens: &Vec<Meal> = &Vec::new();
    for canteen in meals.keys() {
        if canteen.canteen == "Mensa Berliner Tor" {
            canteens = meals.get(canteen).unwrap();
            let now: DateTime<Utc> = Utc::now();

            if canteen.date.format("%d.%m.%Y").to_string() != now.format("%d.%m.%Y").to_string() {
                println!("No meals for today");
                return;
            }

            break;
        }
    }
    println!("{:#?}", canteens);

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            meals: canteens.to_vec(),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

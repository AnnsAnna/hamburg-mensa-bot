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
        let messages = channel
            .messages(&ctx.http, |retriever| retriever.limit(100))
            .await
            .unwrap();
        if messages.len() > 0 {
            channel.delete_messages(&ctx.http, messages).await.unwrap();
        }

        if self.meals.is_none() {
            channel
                .send_message(&ctx.http, |m| m.content("No meals today :("))
                .await
                .unwrap();
        } else {
            let meals = self.meals.as_ref().unwrap();

            channel.send_message(&ctx.http, |m| {
                meals.iter().for_each(|meal| {
                    m.add_embed(|embed| {
                        embed.title(&meal.name)
                        .description(emojify_contents(&meal.contents))
                        .field("Preis Student", format!("{}€", &meal.prices.price_student), true)
                        .field("Preis Mitarbeiter", format!("{}€", &meal.prices.price_attendant), true)
                        .field("Preis Gast", format!("{}€", &meal.prices.price_guest), true)
                        .field("Kategorie", &meal.category, true)
                        .footer();
                        embed
                    });
                });
                // m.add_embed(|embed| {
                //     embed.title("Allergen")
                //     .field("Ei", "Ei und Eierzeugnisse", true)
                //     .field("En", "Erdnüsse und Erdnusserzeugnisse", true)
                //     .field("Fi", "Fisch und Fischerzeugnisse", true)
                //     .field("Gl", "glutenhaltiges Getreide und daraus hergestellte Erzeugnisse (z. B. Weizen, Roggen, Gerste etc.)", true)
                //     .field("Kr", "Krebstiere und Krebstiererzeugnisse", true)
                //     .field("La", "Milch und Milcherzeugnisse (einschl. Laktose)", true)
                //     .field("Lu", "Lupine und - erzeugnisse", true)
                //     .field("Nu", "Schalenfrüchte (z.B. Mandel, Haselnüsse, Walnuss etc.)/-erzeugnisse", true)
                //     .field("Se", "Sesamsamen und Sesamsamenerzeugnisse", true)
                //     .field("Sf", "Senf und Senferzeugnisse", true)
                //     .field("Sl", "Sellerie und Sellerieerzeugnisse", true)
                //     .field("So", "Soja und Sojaerzeugnisse", true)
                //     .field("Sw", "Schwefeldioxid und Sulfite (Konzentration über 10mg/kg oder 10mg/l)", true)
                //     .field("Wt", "Weichtiere (z.B. Muscheln und Weinbergschnecken) und Weichtiererzeugnisse", true)
                // });
                // m.add_embed(|embed| {
                //     embed.title("Zusatzstoffe")
                //     .field("1", "mit Farbstoff", true)
                //     .field("2", "mit Konservierungsstoff", true)
                //     .field("3", "mit Antioxidationsmittel", true)
                //     .field("4", "mit Geschmacksverstärker", true)
                //     .field("5", "Geschwefelt", true)
                //     .field("6", "Geschwärzt", true)
                //     .field("7", "Gewachst", true)
                //     .field("8", "mit Phosphat", true)
                //     .field("9", "mit Süßungsmittel", true)
                // })
                m
            }).await.unwrap();
        }
    }
}

fn emojify_contents(content: &meals::Contents) -> String {
    // Return a string of emojis for the contents of the meal.
    let mut emojis = String::new();
    if content.alcohol {
        emojis.push_str("🍷Alkohol ");
    }
    if content.beef {
        emojis.push_str("🐄💀Fleisch ");
    }
    if content.fish {
        emojis.push_str("🐟💀Fisch ");
    }
    if content.game {
        emojis.push_str("🦌💀Fleisch ");
    }
    if content.gelatine {
        emojis.push_str("🐖💀Fleisch ");
    }
    if !content.lactose_free {
        emojis.push_str("🥛Laktose ");
    }
    if content.lamb {
        emojis.push_str("🐑💀Fleisch ");
    }
    if content.pig {
        emojis.push_str("🐖💀Fleisch ");
    }
    if content.poultry {
        emojis.push_str("🐓💀Fleisch ");
    }
    if content.vegan {
        emojis.push_str("🌱Vegan ");
    }
    if content.vegetarian {
        emojis.push_str("🥕Vegetarisch ");
    }
    emojis
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
        .get(format!("{}{}.json", BASE_URL, now.format("%Y/%m/22")))
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

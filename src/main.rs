

use std::env;

use regex::Regex;
use serenity::async_trait;

use serenity::model::gateway::Ready;
use serenity::prelude::*;
extern crate dotenv;




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
        let bracket_regex = Regex::new(r"\(\s?[^)]*[,]{1}[^)]*\)|\([^)]{1,2}\)").unwrap();

        // Clear the channel
        let messages = channel
            .messages(&ctx.http, |retriever| retriever.limit(100))
            .await
            .unwrap();
        if !messages.is_empty() {
            channel.delete_messages(&ctx.http, messages).await.unwrap();
        }

        if self.meals.is_none() {
            channel
                .send_message(&ctx.http, |m| m.content("No meals today :("))
                .await
                .unwrap();
        } else {
            let meals = self.meals.as_ref().unwrap();

            channel
                .send_message(&ctx.http, |m| {
                    meals.iter().for_each(|meal| {

                        // Remove brackets from the meal name including it's content
                        let result = bracket_regex.replace_all(&meal.name, "");

                        m.add_embed(|embed| {
                            embed
                                .title(result)
                                .description(emojify_contents(&meal.contents))
                                .field("Kategorie", &meal.category, true)
                                .field(
                                    "Preis Student",
                                    format!("**{}€**", &meal.prices.price_student),
                                    true,
                                )
                                .field(
                                    "Preis Mitarbeiter",
                                    format!("{}€", &meal.prices.price_attendant),
                                    true,
                                )
                                // .field("Preis Gast", format!("{}€", &meal.prices.price_guest), true)
                                .footer(|footer| {
                                    footer.text(
                                        &meal
                                            .additives.values().map(|v| v.to_string())
                                            .collect::<Vec<String>>()
                                            .join(", "),
                                    )
                                });
                            if meal.contents.vegan {
                                embed.color(0x70bf1e);
                            } else if meal.contents.vegetarian {
                                embed.color(0xc8d827);
                            } else {
                                embed.color(0x592323);
                            }
                            embed
                        });
                    });
                    m
                })
                .await
                .unwrap();
        }
    }
}

fn emojify_contents(content: &meals::Contents) -> String {
    // Return a string of emojis for the contents of the meal.
    let mut emojis = String::new();
    if content.vegan {
        emojis.push_str("🌱Vegan ");
    }
    if content.vegetarian {
        emojis.push_str("🥕Vegetarisch ");
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
    if content.lamb {
        emojis.push_str("🐑💀Fleisch ");
    }
    if content.pig {
        emojis.push_str("🐖💀Fleisch ");
    }
    if content.poultry {
        emojis.push_str("🐓💀Fleisch ");
    }
    if !content.lactose_free {
        emojis.push_str("🥛Laktose ");
    }
    if content.alcohol {
        emojis.push_str("🍷Alkohol ");
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
        .get(format!("{}{}.json", BASE_URL, now.format("%Y/%m/25")))
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
            meals,
            channel_id: channel_id.parse::<u64>().unwrap(),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

use std::env;
use std::ops::Add;

use chrono::Datelike;
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
    is_in_future: bool,
}

const BASE_URL: &str =
    "https://raw.githubusercontent.com/HAWHHCalendarBot/mensa-data/main/";
const USER_AGENT: &str = "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com) - HAW Mensa Bot";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let channel = ctx.http.get_channel(self.channel_id).await.unwrap().id();
        // This god awful regex should clear it from non-important brackets
        let bracket_regex = Regex::new(r"\s?(\([^)]*[,]{1}[^)]*\)|\([^)]{1,2}\))").unwrap();

        // Clear the channel
        let messages = channel
            .messages(&ctx.http, |retriever| retriever.limit(100))
            .await
            .unwrap();
        if !messages.is_empty() {
            channel.delete_messages(&ctx.http, messages).await.unwrap();
        }

        if self.meals.is_none() {
            // This shouldn't happen in reality unless there is no plan whatsoever, but just in case
            channel
                .send_message(&ctx.http, |m| m.content("Can't find any meals :("))
                .await
                .unwrap();
        } else {
            let meals = self.meals.as_ref().unwrap();
            let parsed_date =
                chrono::NaiveDate::parse_from_str(&meals[0].date, "%Y-%m-%d").unwrap();

            channel
                .send_message(&ctx.http, |m| {
                    if self.is_in_future {
                        m.add_embed(|embed| {
                          embed.color(0xff0000).title("🚨Achtung: Dieser Plan ist für die Zukunft!")
                        });
                    }

                    m.add_embed(|embed| {
                        embed
                            .title(parsed_date.format_localized("%A, %-d %B in der Mensa gibt es:", chrono::Locale::de_DE))
                            .footer(|f| f.text("❤️EdJoPaTo für mensa-crawler!\n 🕵️Quellcode: https://github.com/AnnsAnna/hamburg-mensa-bot"))
                            .color(0x00ff00)
                    });
                    meals.iter().for_each(|meal| {
                        // Remove brackets from the meal name including it's content
                        let result = bracket_regex.replace_all(&meal.name, "");

                        m.add_embed(|embed| {
                            embed
                                .title(result)
                                .description(emojify_contents(&meal.contents))
                                .field("Kategorie", &meal.category, true)
                                .field(
                                    "**Preis Student**",
                                    format!("**__{}€__**", &meal.prices.price_student),
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
                                            .additives
                                            .values()
                                            .map(|v| v.to_string())
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
        emojis.push_str("🌱 Vegan ");
    }
    if content.vegetarian {
        emojis.push_str("🥕 Vegetarisch ");
    }
    if content.beef {
        emojis.push_str("🐄💀 Fleisch ");
    }
    if content.fish {
        emojis.push_str("🐟💀 Fisch ");
    }
    if content.game {
        emojis.push_str("🦌💀 Fleisch ");
    }
    if content.gelatine {
        emojis.push_str("🐖💀 Fleisch ");
    }
    if content.lamb {
        emojis.push_str("🐑💀 Fleisch ");
    }
    if content.pig {
        emojis.push_str("🐖💀 Fleisch ");
    }
    if content.poultry {
        emojis.push_str("🐓💀 Fleisch ");
    }
    if !content.lactose_free {
        emojis.push_str("🥛 Laktose ");
    }
    if content.alcohol {
        emojis.push_str("🍷 Alkohol ");
    }
    emojis
}

fn meal_weight(content: &meals::Contents) -> u8 {
    if content.vegan {
        1
    } else if content.vegetarian {
        2
    } else {
        3
    }
}

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file.
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id = env::var("CHANNEL_ID").expect("Expected a channel id in the environment");
    let mensa = env::var("MENSA").expect("Expected a mensa in the environment");

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap();

    let mut now = chrono::Local::now();

    // Change now to Monday if it's Saturday or Sunday
    let is_in_future = if now.weekday() == chrono::Weekday::Sat {
        now = now.add(chrono::Duration::days(2));
        true
    } else if now.weekday() == chrono::Weekday::Sun {
        now = now.add(chrono::Duration::days(1));
        true
    } else {
        false
    };

    let request = client
        .get(format!("{}/{}/{}.json", BASE_URL, mensa, now.format("%Y/%m/%d")))
        .send()
        .await
        .unwrap();

    println!("Status: {}", request.status());

    let meals = if request.status() == 404 {
        None
    } else {
        let mut parsed = request.json::<Vec<meals::Meal>>().await.unwrap();
        // Sort by content with vegan as highest priority
        parsed.sort_by(|a, b| {
            let a = meal_weight(&a.contents);
            let b = meal_weight(&b.contents);
            a.cmp(&b)
        });
        Some(parsed)
    };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            meals,
            channel_id: channel_id.parse::<u64>().unwrap(),
            is_in_future,
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

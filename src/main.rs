use std::env;
use std::ops::Add;

use chrono::{Datelike, Timelike};
use meals::Meal;
use regex::Regex;
use serenity::async_trait;

use serenity::model::gateway::Ready;
use serenity::prelude::*;
extern crate dotenv;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

mod meals;

use dotenv::dotenv;
struct Handler {
    meals: Option<Vec<meals::Meal>>,
    channel_id: u64,
    is_in_future: bool,
    mensa: String,
    is_identical: bool,
}

const FUTURE_WARNING: &str = "🚨Achtung: Dieser Plan ist für die Zukunft!";
const NO_PLAN_MESSAGE: &str = "🚨 Es gibt keinen Plan für die Mensa!";
const BASE_URL: &str = "https://raw.githubusercontent.com/HAWHHCalendarBot/mensa-data/main/";
const USER_AGENT: &str =
    "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com) - HAW Mensa Bot";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let channel = ctx.http.get_channel(self.channel_id).await.unwrap().id();

        let messages = channel
            .messages(&ctx.http, |retriever| retriever.limit(100))
            .await
            .unwrap();

        if self.is_identical {
            let message = messages.iter().find(|m| {
                m.embeds
                    .iter()
                    .any(|e| e.title.is_some() && e.title.as_ref().unwrap() == FUTURE_WARNING)
            });
            if message.is_some() {
                channel
                    .delete_message(&ctx.http, message.unwrap().id)
                    .await
                    .unwrap();
            }

            // Close the connection
            ctx.shard.shutdown_clean();
            std::process::exit(0);
        } else if self.meals.is_none() {
            let message = messages.iter().find(|m| {
                m.embeds
                    .iter()
                    .any(|e| e.title.is_some() && e.title.as_ref().unwrap() == NO_PLAN_MESSAGE)
            });
            if message.is_some() {
                // Close the connection
                ctx.shard.shutdown_clean();
                std::process::exit(0);
            }
        }

        // Clear the channel
        if !messages.is_empty() {
            match channel.delete_messages(&ctx.http, messages).await {
                Ok(_) => println!("Deleted messages"),
                Err(e) => println!("Error deleting messages - Presumably too old: {:?}", e),
            }
        }
        if self.is_in_future {
            channel
                .send_message(&ctx.http, |m| {
                    m.add_embed(|embed| embed.color(0xff0000).title(FUTURE_WARNING))
                })
                .await
                .unwrap();
        }

        if self.meals.is_none() {
            channel
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(NO_PLAN_MESSAGE)
                            .description("
                                Dies kann mehrere Gründe haben z.B. \
                                könnte die Mensa auf Grund von Feiertagen oder Sonderveranstaltungen geschlossen sein. \
                                Es kann aber auch sein, dass es eine IT Störung gibt und der Plan nicht geladen werden kann oder \
                                dass der Bot nicht richtig funktioniert.
                                **Informiere dich unter: https://www.stwhh.de/speiseplan**")
                            .footer(|f| f.text("Wenn du denkst, dass dies ein Fehler ist, melde dich unter https://github.com/AnnsAnns/hamburg-mensa-bot/issues/new"))
                            .color(0xff0000)
                    })
                })
                .await
                .unwrap();
        } else {
            let meals = self.meals.as_ref().unwrap();
            // Discord has a limit on the amount of embeds per message
            let mut meal_chunks = meals.chunks(8);
            let parsed_date =
                chrono::NaiveDate::parse_from_str(&meals[0].date, "%Y-%m-%d").unwrap();

            channel
                .send_message(&ctx.http, |m| {
                    m.add_embed(|embed| {
                        embed
                            .title(format!("{} in der {} gibt es:", parsed_date.format_localized("%A, %-d %B", chrono::Locale::de_DE), self.mensa))
                            .footer(|f| f.text("❤️EdJoPaTo für mensa-crawler!\n 🕵️Quellcode: https://github.com/AnnsAnns/hamburg-mensa-bot"))
                            .color(0xf9a2e3)
                    });
                    m
                })
                .await
                .unwrap();

            loop {
                let chunk = meal_chunks.next();

                if chunk.is_none() {
                    break;
                }

                let chunk = chunk.unwrap();

                channel
                    .send_message(&ctx.http, |m| {
                        chunk.iter().for_each(|meal| {
                            add_meal(meal, m);
                        });
                        m
                    })
                    .await
                    .unwrap();
            }
        }

        // Close the connection
        ctx.shard.shutdown_clean();
        std::process::exit(0);
    }
}

fn add_meal(meal: &Meal, m: &mut serenity::builder::CreateMessage) {
    // This god awful regex should clear it from non-important brackets
    let bracket_regex = Regex::new(r"\s?(\([^)]*[,]{1}[^)]*\)|\([^)]{1,2}\))").unwrap();

    // Remove brackets from the meal name including it's content
    let result = bracket_regex.replace_all(&meal.name, "");

    m.add_embed(|embed| {
        embed
            .title(result)
            .description(emojify_contents(&meal.contents))
            .field("Kategorie", &meal.category, true)
            .field(
                "**Preis Student**",
                format!("**__{:.2}€__**", &meal.prices.price_student),
                true,
            )
            .field(
                "Preis Mitarbeiter",
                format!("{:.2}€", &meal.prices.price_attendant),
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
    let mut is_identical = false;

    // Change now to Monday if it's Saturday or Sunday
    let is_in_future = if now.weekday() == chrono::Weekday::Sat {
        now = now.add(chrono::Duration::days(2));
        true
    } else if now.weekday() == chrono::Weekday::Sun {
        now = now.add(chrono::Duration::days(1));
        true
    } else if now.hour() >= 18 {
        if now.weekday() == chrono::Weekday::Fri {
            now = now.add(chrono::Duration::days(3));
        } else {
            now = now.add(chrono::Duration::days(1));
        }
        true
    } else {
        false
    };

    let request_url = format!("{}{}/{}.json", BASE_URL, mensa, now.format("%Y/%m/%d"));

    println!("Requesting: {}", request_url);

    let request = client
        .get(&request_url)
        .send()
        .await
        .unwrap();

    println!("Status: {} @ {}", request.status(), request.url());

    let meals = if request.status() == 404 {
        None
    } else {
        let mut parsed = request.json::<Vec<meals::Meal>>().await.unwrap();

        // Save to file
        let file = File::open(format!("{}.json", mensa)).await;

        is_identical = if file.is_err() {
            false
        } else {
            let mut file = file.unwrap();
            let mut old_meals = String::new();
            file.read_to_string(&mut old_meals).await.unwrap();
            let old_meals: Vec<meals::Meal> = serde_json::from_str(&old_meals).unwrap();

            old_meals == parsed
        };

        if !is_identical {
            let mut file = File::create(format!("{}.json", mensa)).await.unwrap();
            file.write_all(serde_json::to_string_pretty(&parsed).unwrap().as_bytes())
                .await
                .unwrap();
        }

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
            mensa,
            is_identical,
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

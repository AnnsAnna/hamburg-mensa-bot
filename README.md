# Hamburg Mensa Bot 🤖

This bot is intended to be run through a daily cronjob. It will post the menu of the day to a discord channel.

![Discord_1KweG8GUKD](https://github.com/AnnsAnns/hamburg-mensa-bot/assets/25822956/7aa864a5-e97f-418f-9689-88784f116546)

## Setup 🛠️

Look at the ".env.template" file and create a ".env" file with the same structure. Alternatively, you can set the environment variables directly.
#### Note
- You can get the discord token from the discord developer portal. 
- The channel id can be obtained by right clicking on the channel in discord and selecting "Copy ID".
- The Mensa URL should be identical to the ones in the [Speiseplan](https://www.stwhh.de/speiseplan?t=this_week)
- Make sure that the bot has the "Manage Messages" permission in the channel you want to post to.
- It will delete all messages in the channel before posting the new menu. **Do not use this bot in a channel where you want to keep messages!**

## Run 🏃

```bash
cargo run
```

## Build 🏗️

```bash
cargo build --release
```

## License 📜

This project is licensed under the EUPL license - see the [LICENSE](LICENSE) file for details

## Acknowledgments and Thanks 🙏
- [EdJoPaTo](https://github.com/EdJoPaTo) for mensa-crawler!

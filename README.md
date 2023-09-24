# Hamburg Mensa Bot 🤖

This bot is intended to be run through a daily cronjob. It will post the menu of the day to a discord channel.

## Here be Dragons 🐲

This was a side-side-side project of mine. If you want to use it, you are free to do so. If you want to improve it, feel free to open a pull request.

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

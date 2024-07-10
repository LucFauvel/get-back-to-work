use dotenv::dotenv;
use serenity::{
    all::{ChannelId, GuildId},
    prelude::*,
};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let bot_time_str = env::var("BOT_TIME").expect("Expected a bot time");
    let bot_time = bot_time_str
        .parse::<u64>()
        .expect("Expected a number for bot time");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .await
        .expect("Err creating client");

    println!("starting a timer with interval of {} seconds", bot_time);

    let http = client.http.clone();
    let cache = client.cache.clone();
    tokio::spawn(async move {
        loop {
            //Pick someone at random here
            sleep(Duration::from_secs(bot_time)).await;

            match http.get_guild(GuildId::new(1221093431364026438)).await {
                Ok(guild) => {
                    if let Ok(channels) = guild.channels(&http).await {
                        if let Some(channel) = channels.get(&ChannelId::new(1221093432274194457)) {
                            println!("channel: {:?}", channel.members(&cache));
                        }
                    }
                }
                Err(why) => {
                    println!("Error getting guild: {:?}", why);
                }
            }
        }
    });

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

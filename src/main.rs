use dotenv::dotenv;
use serenity::{
    all::{Cache, ChannelId, GuildId, Http, Ready},
    async_trait,
    prelude::*,
};
use std::env;
use tokio::{
    sync::oneshot::{self, Receiver, Sender},
    time::{sleep, Duration},
};

pub const GUILD_ID: u64 = 1221093432274194457;
async fn get_current_channel_users(http: &Http, cache: &Cache) {
    match http.get_guild(GuildId::new(1221093431364026438)).await {
        Ok(guild) => {
            if let Ok(channels) = guild.channels(&http).await {
                if let Some(channel) = channels.get(&ChannelId::new(GUILD_ID)) {
                    println!("channel: {:?}", channel.members(&cache));
                }
            }
        }
        Err(why) => {
            println!("Error getting guild: {:?}", why);
        }
    }
}

struct ReadyOneshot {
    pub sender: Sender<bool>,
    pub reader: Receiver<bool>
}

struct Handler<'a> {
    pub ready_oneshot : &'a mut ReadyOneshot
}

#[async_trait]
impl<'a> EventHandler for Handler<'a> {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("ready af");
        // get_current_channel_users(http, cache)
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (tx, rx) = oneshot::channel::<bool>();

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


    let mut ready_oneshot = ReadyOneshot {
        sender: tx,
        reader: rx
    };

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler{ ready_oneshot: &mut ready_oneshot })
        .await
        .expect("Err creating client");

    println!("starting a timer with interval of {} seconds", bot_time);

    let http = client.http.clone();
    let cache = client.cache.clone();
    tokio::spawn(async move {
        let _ = rx.await;

        loop {
            get_current_channel_users(&http, &cache).await;

            //Pick someone at random here
            sleep(Duration::from_secs(bot_time)).await;
        }
    });

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

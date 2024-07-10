use dotenv::dotenv;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;

use serenity::{
    all::{
        Cache, Channel, ChannelId, CreateInteractionResponse, CreateMessage, EditMember, GuildId, Http, MessageBuilder, Ready
    },
    async_trait,
    futures::channel::oneshot::{channel, Sender},
    prelude::*,
};
use std::env;
use tokio::time::{sleep, Duration};

pub const GUILD_ID: u64 = 1221093432274194457;
async fn get_current_channel_users(http: &Http, cache: &Cache) {
    match http.get_guild(GuildId::new(1221093431364026438)).await {
        Ok(guild) => {
            if let Ok(channels) = guild.channels(&http).await {
                if let Some(channel) = channels.get(&ChannelId::new(GUILD_ID)) {
                    if let Ok(members) = channel.members(&cache) {
                        if let Some(random_user) = members.choose(&mut OsRng) {
                            let mut random_user = random_user.clone();
                            let builder = CreateMessage::new().content(format!(
                                "We are going to make <@{}>",
                                random_user.user.id.to_string()
                            ));

                            let _ = ChannelId::new(1221112008800469052)
                                .send_message(http, builder)
                                .await;

                            random_user.edit(http, EditMember::new().mute(true).deafen(true)).await;

                            
                        }
                    }
                }
            }
        }
        Err(why) => {
            println!("Error getting guild: {:?}", why);
        }
    }
}

struct ReadyOneshotSender;

impl TypeMapKey for ReadyOneshotSender {
    type Value = Sender<bool>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        if let Some(tx) = ctx.data.write().await.remove::<ReadyOneshotSender>() {
            let _ = tx.send(true);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (tx, rx) = channel::<bool>();

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
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    client.data.write().await.insert::<ReadyOneshotSender>(tx);

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

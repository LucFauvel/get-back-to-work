use dotenv::dotenv;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;

use serenity::{
    all::{Cache, ChannelId, CreateMessage, EditMember, GuildId, Http, Ready},
    async_trait,
    futures::channel::oneshot::{channel, Sender},
    model::error::Error as SerenityModelError,
    prelude::*,
    Error as SerenityError,
};
use std::env;
use tokio::time::{sleep, Duration};

pub const GUILD_ID: u64 = 1221093431364026438;
pub const VOICE_CHANNEL_ID: u64 = 1221093432274194457;
pub const TEXT_CHANNEL_ID: u64 = 1221112008800469052;

async fn mute_random_user(http: &Http, cache: &Cache) -> Result<(), SerenityError> {
    let mut channel_members = http
        .get_guild(GuildId::new(GUILD_ID))
        .await?
        .channels(http)
        .await?
        .get(&ChannelId::new(VOICE_CHANNEL_ID))
        .ok_or(SerenityModelError::ChannelNotFound)?
        .members(cache)?;

    let random_channel_member = channel_members
        .choose_mut(&mut OsRng)
        .ok_or(SerenityModelError::MemberNotFound)?;

    let builder = CreateMessage::new().content(format!(
        "We are going to mute <@{}>",
        random_channel_member.user.id
    ));

    ChannelId::new(TEXT_CHANNEL_ID)
        .send_message(http, builder)
        .await?;

    random_channel_member
        .edit(http, EditMember::new().mute(true).deafen(true))
        .await?;

    Ok(())
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
    let _ = dotenv();
    let (tx, rx) = channel::<bool>();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let bot_time_str = env::var("BOT_TIME").expect("Expected a bot time");
    let bot_time = bot_time_str
        .parse::<u64>()
        .expect("Expected a number for bot time");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    client.data.write().await.insert::<ReadyOneshotSender>(tx);

    println!("starting a timer with interval of {} seconds", bot_time);

    let http = client.http.clone();
    let cache = client.cache.clone();
    tokio::spawn(async move {
        let _ = rx.await;
        loop {
            if let Err(why) = mute_random_user(&http, &cache).await {
                println!("Error muting random user: {why:?}");
            }

            sleep(Duration::from_secs(bot_time)).await;
        }
    });

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

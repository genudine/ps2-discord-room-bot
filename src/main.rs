use config::WATCH_CHANNELS;
use serenity::{
    async_trait,
    model::{
        prelude::{ChannelId, ChannelType, Ready},
        voice::VoiceState,
    },
    prelude::*,
};
use std::env;
use tracing::{debug, log::trace};

mod config;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, _event: Ready) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(150)).await;
            trace!("pruning channels");
            for (guild_id, channel_id) in WATCH_CHANNELS.iter() {
                trace!("Pruning channels for guild {:?}", guild_id);
                prune_channels(context.clone(), channel_id).await;
            }
        }
    }
    async fn voice_state_update(
        &self,
        context: Context,
        old_voice_state: Option<VoiceState>,
        voice_state: VoiceState,
    ) {
        trace!("Voice state update: {:?}", voice_state);
        trace!("Old state: {:?}", old_voice_state);

        if voice_state.guild_id.is_none() {
            debug!("No guild id...");
            return;
        }

        let guild_id = voice_state.guild_id.unwrap().to_string();
        let trigger_channel_id = match WATCH_CHANNELS.get(&guild_id) {
            Some(channel) => channel,
            None => {
                debug!("No trigger channel for guild {:?}", guild_id);
                return;
            }
        };
        debug!(
            "Trigger channel for guild {:?}: {:?}",
            guild_id, trigger_channel_id
        );

        if voice_state.channel_id.is_none() {
            debug!("No channel id, user left. Running prune");
            prune_channels(context, trigger_channel_id).await;
            return;
        }

        if old_voice_state.is_some() {
            let old_channel_id = old_voice_state.unwrap().channel_id;
            if old_channel_id.unwrap().to_string() == *trigger_channel_id {
                debug!("old == trigger channel, assuming left channel");
            } else {
                let new_channel_id = voice_state.channel_id;

                if old_channel_id != new_channel_id {
                    debug!("user changed channels, running prune");
                    tokio::spawn(prune_channels(context.clone(), trigger_channel_id));
                }
            }
        }

        let channel_id = voice_state.channel_id.unwrap();
        if channel_id.to_string() != *trigger_channel_id {
            debug!("Not the trigger channel");
            return;
        }

        debug!("Trigger channel hit, creating a channel");
        create_room(context, voice_state, trigger_channel_id).await;
    }
}

async fn create_room(context: Context, voice_state: VoiceState, trigger_channel_id: &String) {
    let member = voice_state.member.unwrap();
    let user_id = voice_state.user_id;
    let guild_id = voice_state.guild_id.unwrap().to_string();
    debug!(
        "Trigger channel for guild {:?}: {:?}",
        guild_id, trigger_channel_id
    );

    let channel_id = voice_state.channel_id.unwrap();
    if channel_id.to_string() != *trigger_channel_id {
        debug!("Not the trigger channel");
        return;
    }

    debug!("Trigger channel hit, creating a channel");

    let guild = match voice_state
        .guild_id
        .unwrap()
        .to_partial_guild(&context.http)
        .await
    {
        Ok(guild) => guild,
        Err(_) => {
            debug!("No guild found");
            return;
        }
    };

    let trigger_channel = channel_id
        .to_channel(&context.http)
        .await
        .expect("Error getting trigger channel")
        .guild()
        .expect("Error getting GuildChannel from trigger Channel");

    let category_id = trigger_channel.parent_id.expect("No category id");

    let channel_name = format!("{}'s room", member.display_name());
    debug!("Channel name: {:?}", channel_name);

    let channel = match guild
        .create_channel(&context.http, |c| {
            c.name(channel_name)
                .kind(ChannelType::Voice)
                .category(category_id)
        })
        .await
    {
        Ok(channel) => channel,
        Err(why) => {
            debug!("Error creating channel: {:?}", why);
            return;
        }
    };

    debug!(
        "Channel created ({:?}), moving user {:?} to it...",
        channel.id, user_id
    );

    member
        .move_to_voice_channel(&context.http, channel.id)
        .await
        .expect("failed to move user to channel");
}

async fn prune_channels(context: Context, trigger_channel_id: &String) {
    trace!("Pruning channels");
    let channel = ChannelId(
        trigger_channel_id
            .parse::<u64>()
            .expect("trigger_channel_id didn't parse..."),
    )
    .to_channel(&context.http)
    .await
    .expect("couldn't get channel")
    .guild()
    .expect("couldn't get guild channel");

    let category = channel
        .parent_id
        .expect("No parent id")
        .to_channel(&context.http)
        .await
        .expect("Couldn't get category channel")
        .category()
        .expect("not a category");

    let channels = category
        .guild_id
        .channels(&context.http)
        .await
        .expect("couldn't get channels");

    let channels = channels.values().filter(|c| {
        c.kind == ChannelType::Voice
            && c.parent_id.is_some()
            && c.parent_id.expect("no parent_id") == category.id
    });

    for channel in channels {
        if channel.id.to_string() == *trigger_channel_id {
            debug!("Skipping trigger channel");
            continue;
        }

        let members = channel
            .members(&context.cache)
            .await
            .expect("couldn't get members");

        if members.len() == 0 {
            debug!("Deleting channel {:?}", channel.id);
            channel
                .delete(&context.http)
                .await
                .expect("couldn't delete channel");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

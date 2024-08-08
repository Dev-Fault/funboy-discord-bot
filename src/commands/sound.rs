use super::*;
use crate::HttpClient;
use io_util::MessageHelper;
use poise::serenity_prelude::async_trait;
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    input::YoutubeDl,
};

use crate::{Context, Error, HttpKey};

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

// Joins bot to current voice channel
#[poise::command(slash_command, prefix_command)]
pub async fn join_voice(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say_ephemeral("Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);

        ctx.say_ephemeral("Joined voice channel").await?;
    }

    Ok(())
}

// Disconnects bot from voice channel
#[poise::command(slash_command, prefix_command)]
pub async fn leave_voice(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            ctx.say(format!("Failed: {:?}", e)).await?;
        }

        ctx.say_ephemeral("Left voice channel").await?;
    } else {
        ctx.say_ephemeral("Not in a voice channel").await?;
    }

    Ok(())
}

// Plays audio from url
//
// Example usage: **/play_url** url: **https://www.youtube.com/watch?v=a3mxLL7nX1E**
#[poise::command(slash_command, prefix_command)]
pub async fn play_url(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let do_search = !url.starts_with("http");

    let guild_id = ctx.guild_id().unwrap();

    let http_client: HttpClient = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let mut src = if do_search {
            YoutubeDl::new_search(http_client, url)
        } else {
            YoutubeDl::new(http_client, url)
        };
        let _ = handler.play_input(src.clone().into());

        ctx.say("Playing song").await?;
    } else {
        ctx.say("Not in a voice channel to play in").await?;
    }

    Ok(())
}

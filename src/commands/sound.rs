use super::*;
use std::sync::Arc;

use crate::{Data, HttpClient};
use io_util::MessageHelper;
use poise::{serenity_prelude::async_trait, CreateReply};
use serenity::all::{
    CacheHttp, ComponentInteraction, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    input::YoutubeDl,
    tracks::TrackHandle,
    Songbird,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{Context, Error, HttpKey};

const NOT_INITIALIZED: &str = "Songbird Voice client placed in at initialisation.";
const NOT_IN_VOICE_CHANNEL: &str = "Not in a voice channel.";
pub const TRACK_BUTTON_ID: &str = "track";
pub const TRACK_OPTIONS: [&str; 3] = ["Play", "Pause", "Stop"];

pub enum TrackOption {
    Play,
    Pause,
    Stop,
    Back,
    Forward,
    Loop,
}

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub handle: TrackHandle,
}

type TrackMap = HashMap<Uuid, Track>;

#[derive(Debug)]
pub struct TrackList {
    track_map: TrackMap,
}

impl TrackList {
    pub fn new() -> Self {
        TrackList {
            track_map: TrackMap::new(),
        }
    }

    pub async fn add_track(&mut self, track: Track) {
        self.clean().await;
        self.track_map.insert(track.handle.uuid(), track);
    }

    pub async fn get_tracks(&mut self) -> Vec<&mut Track> {
        self.clean().await;
        self.track_map.values_mut().collect()
    }

    pub async fn get_track(&mut self, id: &str) -> Option<&mut Track> {
        self.clean().await;
        for (uuid, track) in &mut self.track_map {
            if uuid.to_string().eq(id) {
                return Some(track);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.track_map.clear();
    }

    async fn clean(&mut self) {
        let mut tracks_to_clean = Vec::new();
        for (_, track) in &self.track_map {
            if let Ok(info) = track.handle.get_info().await {
                match info.playing {
                    songbird::tracks::PlayMode::Play => {}
                    songbird::tracks::PlayMode::Pause => {}
                    _ => tracks_to_clean.push(track.handle.uuid()),
                }
            } else {
                tracks_to_clean.push(track.handle.uuid());
            }
        }

        for track in tracks_to_clean {
            self.track_map.remove(&track);
        }
    }
}

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}.",
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
            ctx.say(NOT_IN_VOICE_CHANNEL).await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect(NOT_INITIALIZED)
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);

        ctx.say("Joined voice channel.").await?;
    }

    Ok(())
}

// Disconnects bot from voice channel
#[poise::command(slash_command, prefix_command)]
pub async fn leave_voice(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().track_list.lock().await.clear();

    let guild_id = ctx.guild_id().unwrap();

    let manager = get_songbird_manager(ctx).await;

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            ctx.say(format!("Failed: {:?}", e)).await?;
        }

        ctx.say("Left voice channel.").await?;
    } else {
        ctx.say(NOT_IN_VOICE_CHANNEL).await?;
    }

    Ok(())
}

// Plays audio track from url or search query
//
// Example usage: **/play_track** url_or_query: **https://www.youtube.com/watch?v=a3mxLL7nX1E**
#[poise::command(slash_command, prefix_command)]
pub async fn play_track(ctx: Context<'_>, url_or_query: String) -> Result<(), Error> {
    let is_url = !url_or_query.starts_with("http");

    let http_client = get_http_client(ctx).await;

    let manager = get_songbird_manager(ctx).await;

    if let Some(handler_lock) = manager.get(ctx.guild_id().unwrap()) {
        let mut handler = handler_lock.lock().await;

        let src = if is_url {
            YoutubeDl::new_search(http_client, url_or_query.clone())
        } else {
            YoutubeDl::new(http_client, url_or_query.clone())
        };

        let track_handle = handler.play_input(src.clone().into());
        ctx.data()
            .track_list
            .lock()
            .await
            .add_track(Track {
                name: url_or_query.clone(),
                handle: track_handle,
            })
            .await;

        ctx.say(format!("Playing track **{}**", &url_or_query))
            .await?;
    } else {
        ctx.say(NOT_IN_VOICE_CHANNEL).await?;
    }

    Ok(())
}

// Stops all currently playing audio tracks
#[poise::command(slash_command, prefix_command)]
pub async fn stop_tracks(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().track_list.lock().await.clear();

    let manager = get_songbird_manager(ctx).await;

    if let Some(handler_lock) = manager.get(ctx.guild_id().unwrap()) {
        let mut handler = handler_lock.lock().await;
        handler.stop();

        ctx.say("Stopping all audio.").await?;
    } else {
        ctx.say(NOT_IN_VOICE_CHANNEL).await?;
    }

    Ok(())
}

pub async fn on_track_button_click(
    ctx: &poise::serenity_prelude::Context,
    component_interaction: &ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let track_id = component_interaction
        .data
        .custom_id
        .split_whitespace()
        .nth(1)
        .unwrap();

    let option = component_interaction
        .data
        .custom_id
        .split_whitespace()
        .nth(2)
        .unwrap();

    if let Some(track) = data.track_list.lock().await.get_track(track_id).await {
        match option {
            "Play" => {
                let _ = track.handle.play();
            }
            "Pause" => {
                let _ = track.handle.pause();
            }
            "Stop" => {
                let _ = track.handle.stop();
            }
            _ => {}
        }
        component_interaction
            .create_response(ctx.http(), CreateInteractionResponse::Acknowledge)
            .await?;
    } else {
        component_interaction
            .create_response(
                ctx.http(),
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Error: Couldn't find track.")
                        .ephemeral(true),
                ),
            )
            .await?;
    }

    Ok(())
}

// Lists currently playing audio tracks
#[poise::command(slash_command, prefix_command)]
pub async fn list_tracks(ctx: Context<'_>) -> Result<(), Error> {
    let manager = get_songbird_manager(ctx).await;

    if let Some(handler_lock) = manager.get(ctx.guild_id().unwrap()) {
        let mut _handler = handler_lock.lock().await;

        let mut track_list = ctx.data().track_list.lock().await;
        let tracks = track_list.get_tracks().await;

        if tracks.len() == 0 {
            ctx.say_ephemeral("No tracks are currently playing.")
                .await?;
            return Ok(());
        }

        for track in &tracks {
            let mut buttons = Vec::new();
            for option in TRACK_OPTIONS {
                buttons.push(
                    CreateButton::new(format!(
                        "{} {} {}",
                        TRACK_BUTTON_ID,
                        &track.handle.uuid(),
                        option
                    ))
                    .label(option),
                );
            }
            ctx.send(
                CreateReply::default()
                    .content(format!("Track: **{}**", &track.name))
                    .ephemeral(true)
                    .components(vec![CreateActionRow::Buttons(buttons)]),
            )
            .await?;
        }
    } else {
        ctx.say(NOT_IN_VOICE_CHANNEL).await?;
    }

    Ok(())
}

async fn get_songbird_manager(ctx: Context<'_>) -> Arc<Songbird> {
    songbird::get(ctx.serenity_context())
        .await
        .expect(NOT_INITIALIZED)
        .clone()
}

async fn get_http_client(ctx: Context<'_>) -> HttpClient {
    let client: HttpClient = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };
    client
}

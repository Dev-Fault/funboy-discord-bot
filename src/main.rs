use std::sync::Arc;

use ::serenity::all::{ClientBuilder, FullEvent, GatewayIntents, Interaction};
use reqwest::Client as HttpClient;
use serenity::all::ComponentInteraction;
use songbird::{typemap::TypeMapKey, SerenityInit};
use template_substitution_database::TemplateDatabase;
use tokio::sync::Mutex;

mod commands;
#[allow(dead_code)]
mod interpreter;
mod io_utils;

use commands::sound::{TrackComponent, TrackList, TRACK_BUTTON_ID};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub template_db: Mutex<TemplateDatabase>,
    pub track_list: Arc<Mutex<TrackList>>,
} // User data, which is stored and accessible in all command invocations

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

enum CustomComponent {
    TrackComponent,
    Invalid,
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::miscellaneous::help(),
                commands::miscellaneous::register(),
                commands::miscellaneous::move_bot_pins(),
                commands::miscellaneous::age(),
                commands::random::random_number(),
                commands::random::random_word(),
                commands::text_gen::add(),
                commands::text_gen::add_sub(),
                commands::text_gen::remove_sub(),
                commands::text_gen::remove_subs(),
                commands::text_gen::remove_template(),
                commands::text_gen::rename_template(),
                commands::text_gen::replace_sub(),
                commands::text_gen::generate(),
                commands::text_gen::list(),
                commands::sound::join_voice(),
                commands::sound::leave_voice(),
                commands::sound::play_track(),
                commands::sound::stop_tracks(),
                commands::sound::list_tracks(),
            ],
            event_handler: |ctx, event, _framework_ctx, data| {
                Box::pin(async move {
                    match event {
                        FullEvent::InteractionCreate {
                            interaction: Interaction::Component(component_interaction),
                        } => match get_component_type(component_interaction) {
                            CustomComponent::TrackComponent => {
                                commands::sound::on_track_button_click(
                                    ctx,
                                    TrackComponent::new(component_interaction.clone()),
                                    data,
                                )
                                .await?;
                            }
                            CustomComponent::Invalid => {}
                        },
                        _ => {}
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {
                    template_db: Mutex::new(
                        TemplateDatabase::from_path("funboy.db")
                            .expect("Failed to load funboy database."),
                    ),
                    track_list: Mutex::new(TrackList::new()).into(),
                })
            })
        })
        .build();

    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await;

    client.unwrap().start().await.unwrap();
}

fn get_component_type(component_interaction: &ComponentInteraction) -> CustomComponent {
    if component_interaction
        .data
        .custom_id
        .starts_with(TRACK_BUTTON_ID)
    {
        return CustomComponent::TrackComponent;
    } else {
        return CustomComponent::Invalid;
    }
}

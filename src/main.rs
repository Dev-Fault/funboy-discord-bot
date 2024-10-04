use std::sync::Arc;

use ::serenity::all::{ClientBuilder, FullEvent, GatewayIntents, Interaction};
use io_utils::custom_components::{CustomComponent, TrackComponent};
use reqwest::Client as HttpClient;
use songbird::{typemap::TypeMapKey, SerenityInit};
use storage::template_database::TemplateDatabase;
use tokio::sync::Mutex;

mod commands;
#[allow(dead_code)]
mod interpreter;
mod io_utils;
mod storage;
mod text_interpolator;

use commands::sound::TrackList;

pub const FUNBOY_DB_PATH: &str = "funboy.db";

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub template_db: Mutex<TemplateDatabase>,
    pub track_list: Arc<Mutex<TrackList>>,
    pub imgur_client_id: Arc<String>,
} // User data, which is stored and accessible in all command invocations

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("must have DISCORD_TOKEN");
    let imgur_client_id = std::env::var("IMGUR_CLIENT_ID").expect("must have IMGUR_CLIENT_ID");

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
                commands::text_gen::copy_subs(),
                commands::text_gen::add_sub(),
                commands::text_gen::remove_sub(),
                commands::text_gen::remove_sub_by_id(),
                commands::text_gen::remove_subs(),
                commands::text_gen::remove_subs_by_id(),
                commands::text_gen::remove_template(),
                commands::text_gen::rename_template(),
                commands::text_gen::replace_sub(),
                commands::text_gen::replace_sub_by_id(),
                commands::text_gen::generate(),
                commands::text_gen::list(),
                commands::text_gen::list_ids(),
                commands::text_gen::list_numerically(),
                commands::sound::join_voice(),
                commands::sound::leave_voice(),
                commands::sound::play_track(),
                commands::sound::stop_tracks(),
                commands::sound::show_tracks(),
                commands::image::search_image(),
            ],
            event_handler: |ctx, event, _framework_ctx, data| {
                Box::pin(async move {
                    match event {
                        FullEvent::InteractionCreate {
                            interaction: Interaction::Component(component_interaction),
                        } => match CustomComponent::from(component_interaction) {
                            CustomComponent::TrackComponent => {
                                commands::sound::on_track_button_click(
                                    ctx,
                                    TrackComponent::new(component_interaction.clone()),
                                    data,
                                )
                                .await?;
                            }
                            CustomComponent::None => {}
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
                        TemplateDatabase::from_path(FUNBOY_DB_PATH)
                            .expect("Failed to load funboy database."),
                    ),
                    track_list: Mutex::new(TrackList::new()).into(),
                    imgur_client_id: Arc::new(imgur_client_id),
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

use poise::serenity_prelude::{self as serenity};
use reqwest::Client as HttpClient;
use songbird::{typemap::TypeMapKey, SerenityInit};
use template_substitution_database::TemplateDatabase;
use tokio::sync::Mutex;

mod commands;

pub struct Data {
    pub t_db: Mutex<TemplateDatabase>,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

//test
#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::miscellaneous::help(),
                commands::miscellaneous::register(),
                commands::miscellaneous::random_number(),
                commands::miscellaneous::random_word(),
                commands::miscellaneous::move_bot_pins(),
                commands::miscellaneous::age(),
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
                commands::sound::play_url(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {
                    t_db: Mutex::new(
                        TemplateDatabase::from_path("funboy.db")
                            .expect("Failed to load funboy database."),
                    ),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await;
    client.unwrap().start().await.unwrap();
}

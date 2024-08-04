use poise::serenity_prelude::{self as serenity};
use template_substitution_database::TemplateDatabase;
use tokio::sync::Mutex;

mod commands;

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::age(),
                commands::add(),
                commands::remove_substitutes(),
                commands::remove_template(),
                commands::rename_template(),
                commands::rename_substitute(),
                commands::generate(),
                commands::list(),
                commands::register(),
                commands::random_number(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(commands::Data {
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
        .await;
    client.unwrap().start().await.unwrap();
}

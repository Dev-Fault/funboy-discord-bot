use serenity::all::UserId;

use crate::{
    io_utils::{
        context_extension::ContextExtension, discord_message_format::ellipsize_if_long,
        input_interp::interp_input,
    },
    ollama_generator::ollama_generator::{OllamaSettings, MAX_PREDICT},
    Context, Error, OllamaSettingsMap,
};

const ERROR_OLLAMA_UNAVAILABLE: &str = "Error: Ollama service not available.";

/// Show all available ollama models
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn show_ollama_models(ctx: Context<'_>) -> Result<(), Error> {
    let ollama_generator = ctx.data().ollama_generator.lock().await;
    let models = ollama_generator.get_models().await;
    match models {
        Err(_) => {
            ctx.say_ephemeral(ERROR_OLLAMA_UNAVAILABLE).await?;
        }
        Ok(models) => {
            ctx.say_ephemeral(
                &models
                    .iter()
                    .fold("".to_string(), |names, model| names + &model.name + "\n"),
            )
            .await?;
        }
    }

    Ok(())
}

fn get_ollama_user_settings<'a>(
    ollama_settings_map: &'a mut OllamaSettingsMap,
    user_id: &UserId,
) -> &'a OllamaSettings {
    ollama_settings_map.entry(*user_id).or_default();
    ollama_settings_map.get(user_id).unwrap()
}

fn get_ollama_user_settings_mut<'a>(
    ollama_settings_map: &'a mut OllamaSettingsMap,
    user_id: &UserId,
) -> &'a mut OllamaSettings {
    ollama_settings_map.entry(*user_id).or_default();
    ollama_settings_map.get_mut(user_id).unwrap()
}

/// Show ollama settings
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn show_ollama_settings(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings(&mut ollama_settings_map, &user_id);

    /*let current_model = match ollama_generator.get_current_model().await {
        Some(name) => name,
        None => "None".to_string(),
    };*/

    ctx.say_ephemeral(&format!(
        "Current Model: {}\n{}",
        //current_model,
        "",
        &settings.to_string()
    ))
    .await?;

    Ok(())
}

/// Set the current ollama model
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_model(ctx: Context<'_>, model: String) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    let models = ollama_generator.get_models().await;
    match models {
        Err(_) => {
            ctx.say_ephemeral(ERROR_OLLAMA_UNAVAILABLE).await?;
        }
        Ok(models) => {
            if models
                .iter()
                .map(|model| &model.name)
                .any(|name| *name == model)
            {
                ollama_generator.set_current_model(&model);
                ctx.say_ephemeral(&format!("Set ollama model to: \"{}\"", model))
                    .await?;
            } else {
                ctx.say_ephemeral(&format!(
                    "Error: \"{}\" is not an avialable ollama model.",
                    model
                ))
                .await?;
            }
        }
    }
    Ok(())
}

/// Set ollama model parameters
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_parameters(
    ctx: Context<'_>,
    temperature: Option<f32>,
    repeat_penalty: Option<f32>,
    top_k: Option<u32>,
    top_p: Option<f32>,
) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id);

    if let Some(temperature) = temperature {
        settings.set_temperature(temperature);
    }
    if let Some(repeat_penalty) = repeat_penalty {
        settings.set_repeat_penalty(repeat_penalty);
    }
    if let Some(top_k) = top_k {
        settings.set_top_k(top_k);
    }
    if let Some(top_p) = top_p {
        settings.set_top_p(top_p);
    }
    ctx.say_ephemeral("Ollama parameters updated.").await?;
    Ok(())
}

/// Reset ollama model parameters to default
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn reset_ollama_parameters(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id);

    settings.reset_parameters();
    ctx.say_ephemeral("Ollama parameters reset.").await?;
    Ok(())
}

/// Set the system prompt for ollama
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_system_prompt(
    ctx: Context<'_>,
    system_prompt: String,
) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id);

    settings.set_system_prompt(&system_prompt);
    ctx.say_ephemeral("Ollama system prompt updated.").await?;
    Ok(())
}

/// Reset the system prompt for ollama to default
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn reset_ollama_system_prompt(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id);

    settings.reset_system_prompt();
    ctx.say_ephemeral("Ollama system prompt reset.").await?;
    Ok(())
}

/// Set the template for ollama
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_template(ctx: Context<'_>, template: String) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id);

    settings.set_template(&template);
    ctx.say_ephemeral("Ollama system prompt updated.").await?;
    Ok(())
}

/// Reset the template for ollama to default
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn reset_ollama_template(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id);

    settings.reset_template();
    ctx.say_ephemeral("Ollama template reset.").await?;
    Ok(())
}

/// Set the maximum amount of words ollama can generate per prompt
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_word_limit(ctx: Context<'_>, limit: u16) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
    let settings = get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id);

    if settings.set_output_limit(limit) {
        ctx.say_ephemeral("Ollama parameters updated.").await?;
    } else {
        ctx.say_ephemeral(&format!(
            "Error: Cannot exceed maximum output limit of {}.",
            MAX_PREDICT
        ))
        .await?;
    }
    Ok(())
}

/// Generate an ollama response from prompt
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn generate_ollama(
    ctx: Context<'_>,
    prompt: String,
    model_override: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let user_id = ctx.author().id;
    let mut users = ctx.data().ollama_users.lock().await;

    if users.contains(&user_id) {
        ctx.say_ephemeral("You are already generating a prompt. Please wait until it is finished.")
            .await?;
        return Ok(());
    } else {
        users.insert(user_id);
    }
    drop(users);

    let db_clone = ctx.data().funboy_db.clone();
    let interpreted_prompt = tokio::task::spawn_blocking(move || interp_input(prompt, db_clone))
        .await?
        .await;

    let result: Result<(), Error> = {
        match interpreted_prompt {
            Ok(prompt) => {
                ctx.say(&format!(
                    "Generating prompt: **\"{}\"**",
                    ellipsize_if_long(&prompt, 200)
                ))
                .await?;

                let user_id = ctx.author().id;
                let mut ollama_settings_map = ctx.data().ollama_settings_map.lock().await;
                let settings =
                    get_ollama_user_settings_mut(&mut ollama_settings_map, &user_id).clone();
                drop(ollama_settings_map);
                let ollama_generator = ctx.data().ollama_generator.lock().await;
                let response = ollama_generator
                    .generate(&prompt, settings, model_override)
                    .await;
                match response {
                    Err(e) => {
                        ctx.say_ephemeral(&format!("Error: {}", e)).await?;
                    }
                    Ok(gen_res) => {
                        ctx.say_long(&format!("{}{}", &prompt, gen_res.response), false)
                            .await?;
                    }
                }
                Ok(())
            }
            Err(e) => {
                ctx.say_ephemeral(&format!("Error: {}", &e)).await?;
                Ok(())
            }
        }
    };

    let mut users = ctx.data().ollama_users.lock().await;
    users.remove(&user_id);

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            ctx.say_ephemeral("Error: Ollama generation failed.")
                .await?;
            Ok(())
        }
    }
}

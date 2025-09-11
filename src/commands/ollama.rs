use crate::{
    io_utils::{
        context_extension::ContextExtension, discord_message_format::ellipsize_if_long,
        input_interp::interp_input,
    },
    ollama_generator::ollama_generator::MAX_PREDICT,
    Context, Error,
};

const ERROR_OLLAMA_UNAVAILABLE: &str = "Error: Ollama service not available.";

/// Show all available ollama models
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn show_ollama_models(ctx: Context<'_>) -> Result<(), Error> {
    let ollama_generator = ctx.data().ollama_generator.lock().await;
    let models = ollama_generator.get_models().await;
    match models {
        Err(e) => {
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

/// Show ollama config
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn show_ollama_settings(ctx: Context<'_>) -> Result<(), Error> {
    let ollama_generator = ctx.data().ollama_generator.lock().await;
    let current_model = match ollama_generator.get_current_model().await {
        Some(name) => name,
        None => "None".to_string(),
    };
    ctx.say_ephemeral(&format!(
        "Current Model: {}\n{}",
        current_model,
        &ollama_generator.get_config().to_string()
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
        Err(e) => {
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
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    if let Some(temperature) = temperature {
        ollama_generator.set_temperature(temperature);
    }
    if let Some(repeat_penalty) = repeat_penalty {
        ollama_generator.set_repeat_penalty(repeat_penalty);
    }
    if let Some(top_k) = top_k {
        ollama_generator.set_top_k(top_k);
    }
    if let Some(top_p) = top_p {
        ollama_generator.set_top_p(top_p);
    }
    ctx.say_ephemeral("Ollama parameters updated.").await?;
    Ok(())
}

/// Reset ollama model parameters to default
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn reset_ollama_parameters(ctx: Context<'_>) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.reset_parameters();
    ctx.say_ephemeral("Ollama parameters reset.").await?;
    Ok(())
}

/// Set the system prompt for ollama
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_system_prompt(
    ctx: Context<'_>,
    system_prompt: String,
) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.set_system_prompt(&system_prompt);
    ctx.say_ephemeral("Ollama system prompt updated.").await?;
    Ok(())
}

/// Reset the system prompt for ollama to default
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn reset_ollama_system_prompt(ctx: Context<'_>) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.reset_system_prompt();
    ctx.say_ephemeral("Ollama system prompt reset.").await?;
    Ok(())
}

/// Set the template for ollama
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_template(ctx: Context<'_>, template: String) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.set_template(&template);
    ctx.say_ephemeral("Ollama system prompt updated.").await?;
    Ok(())
}

/// Reset the template for ollama to default
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn reset_ollama_template(ctx: Context<'_>) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.reset_template();
    ctx.say_ephemeral("Ollama template reset.").await?;
    Ok(())
}

/// Set the maximum amount of words ollama can generate per prompt
#[poise::command(slash_command, prefix_command, category = "Ollama")]
pub async fn set_ollama_word_limit(ctx: Context<'_>, limit: u16) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    if ollama_generator.set_output_limit(limit) {
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
    temperature_override: Option<f32>,
    model_override: Option<String>,
) -> Result<(), Error> {
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

    let db = ctx.data().funboy_db.lock().await;
    let db_path = ctx.data().get_template_db_path();
    let interpreted_prompt = interp_input(&prompt, db_path, &|template| match db
        .get_random_subs(template)
    {
        Ok(sub) => Some(sub),
        Err(_) => None,
    });
    drop(db);

    let result: Result<(), Error> = {
        match interpreted_prompt {
            Ok(prompt) => {
                ctx.say(&format!(
                    "Generating prompt: **\"{}\"**",
                    ellipsize_if_long(&prompt, 200)
                ))
                .await?;
                ctx.defer().await?;

                let ollama_generator = ctx.data().ollama_generator.lock().await;
                let response = ollama_generator
                    .generate(&prompt, temperature_override, model_override)
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

use ollama_rs::models;
use poise::CreateReply;

use crate::{
    fsl_interpreter::Interpreter,
    io_utils::context_extension::ContextExtension,
    ollama_generator::{
        self,
        ollama_generator::{OllamaParameters, MAX_PREDICT},
    },
    text_interpolator::TextInterpolator,
    Context, Error,
};

const ERROR_OLLAMA_UNAVAILABLE: &str = "Error: Ollama service not available.";

/// Show all available ollama models
#[poise::command(slash_command, prefix_command)]
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
#[poise::command(slash_command, prefix_command)]
pub async fn show_ollama_config(ctx: Context<'_>) -> Result<(), Error> {
    let ollama_generator = ctx.data().ollama_generator.lock().await;
    ctx.say_ephemeral(&ollama_generator.get_config().to_string())
        .await?;

    Ok(())
}

/// Set the current ollama model
#[poise::command(slash_command, prefix_command)]
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
                ollama_generator.set_selected_model(&model);
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
#[poise::command(slash_command, prefix_command)]
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
#[poise::command(slash_command, prefix_command)]
pub async fn reset_ollama_parameters(ctx: Context<'_>) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.reset_parameters();
    ctx.say_ephemeral("Ollama parameters reset.").await?;
    Ok(())
}

/// Set the system prompt for ollama
#[poise::command(slash_command, prefix_command)]
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
#[poise::command(slash_command, prefix_command)]
pub async fn reset_ollama_system_prompt(ctx: Context<'_>) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.reset_system_prompt();
    ctx.say_ephemeral("Ollama system prompt reset.").await?;
    Ok(())
}

/// Set the template for ollama
#[poise::command(slash_command, prefix_command)]
pub async fn set_ollama_template(ctx: Context<'_>, template: String) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.set_template(&template);
    ctx.say_ephemeral("Ollama system prompt updated.").await?;
    Ok(())
}

/// Reset the template for ollama to default
#[poise::command(slash_command, prefix_command)]
pub async fn reset_ollama_template(ctx: Context<'_>) -> Result<(), Error> {
    let mut ollama_generator = ctx.data().ollama_generator.lock().await;
    ollama_generator.reset_template();
    ctx.say_ephemeral("Ollama template reset.").await?;
    Ok(())
}

/// Set the maximum amount of words ollama can generate per prompt
#[poise::command(slash_command, prefix_command)]
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
#[poise::command(slash_command, prefix_command)]
pub async fn generate_ollama(
    ctx: Context<'_>,
    prompt: String,
    temperature: Option<f32>,
) -> Result<(), Error> {
    let db = ctx.data().template_db.lock().await;
    let mut interpolator = TextInterpolator::default();

    let output = interpolator.interp(&prompt, &|template| match db.get_random_subs(template) {
        Ok(sub) => Some(sub),
        Err(_) => None,
    });

    let mut fsl_interpreter = Interpreter::new();

    let interpreted_prompt: Result<String, String> = match output {
        Ok(output) => match fsl_interpreter.interpret_embedded_code(&output) {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        },
        Err(e) => Err(e.to_string()),
    };

    match interpreted_prompt {
        Ok(prompt) => {
            ctx.say("Generating response...").await?;
            ctx.defer().await?;

            let ollama_generator = ctx.data().ollama_generator.lock().await;
            let response = ollama_generator.generate(&prompt, temperature).await;
            match response {
                Err(_) => {
                    ctx.say_ephemeral(ERROR_OLLAMA_UNAVAILABLE).await?;
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
}

use super::*;
use poise::{serenity_prelude::CreateInteractionResponseMessage, CreateReply};
use rand::Rng;
use template_substitution_database::rusqlite;
use text_interpolator::TextInterpolator;

use io_util::*;

const TEMPLATE_NAME_ERROR: &str = "Error: templates may only contain letters and numbers.";

#[poise::command(slash_command, prefix_command)]
pub async fn add(ctx: Context<'_>, template: String, substitutes: String) -> Result<(), Error> {
    if template.contains(|c: char| !c.is_alphanumeric()) {
        ctx.say(TEMPLATE_NAME_ERROR).await?;
        return Ok(());
    }

    let mut db = ctx.data().t_db.lock().await;

    let subs_to_insert: Vec<&str> = vectorize_input(substitutes.as_str());

    match db.insert_substitutions(&template, Some(&subs_to_insert)) {
        Err(e) => {
            ctx.say(e.to_string()).await?;
        }
        Ok(inserted_subs) => {
            let output_log = OutputLog::from(subs_to_insert, inserted_subs);

            ctx.say(format!(
                "Added substitutes [{}] under template {}.",
                &output_log.present, &template
            ))
            .await?;

            if output_log.not_present.len() > 0 {
                ctx.say(format!(
                    "Substitutes [{}] are already present under template {}.",
                    &output_log.not_present, &template
                ))
                .await?;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn remove_substitutes(
    ctx: Context<'_>,
    template: String,
    substitutes: String,
) -> Result<(), Error> {
    let mut db = ctx.data().t_db.lock().await;

    let subs_to_remove: Vec<&str> = vectorize_input(substitutes.as_str());

    match db.remove_substitutes(&template, &subs_to_remove) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say(format!("Error: No template named {} exists.", &template))
                    .await?;
            }
            _ => {
                ctx.say(e.to_string()).await?;
            }
        },
        Ok(removed_subs) => {
            let output_log = OutputLog::from(subs_to_remove, removed_subs);
            ctx.say(format!(
                "Removed substitutes [{}] from template {}.",
                output_log.present, &template
            ))
            .await?;
            if output_log.not_present.len() > 0 {
                ctx.say(format!(
                    "Substitutes [{}] were not found in template {}.",
                    output_log.not_present, &template
                ))
                .await?;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn remove_template(ctx: Context<'_>, template: String) -> Result<(), Error> {
    let mut db = ctx.data().t_db.lock().await;

    match db.remove_template(&template) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say(format!("Error: No template named {} exists.", &template))
                    .await?;
            }
            _ => {
                ctx.say(e.to_string()).await?;
            }
        },
        Ok(result) => {
            if result {
                ctx.say(format!("Removed template {}.", &template)).await?;
            } else {
                ctx.say(format!("Error: No template named {} exists.", &template))
                    .await?;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn rename_template(ctx: Context<'_>, from: String, to: String) -> Result<(), Error> {
    if from.contains(|c: char| !c.is_alphanumeric()) || to.contains(|c: char| !c.is_alphanumeric())
    {
        ctx.say(TEMPLATE_NAME_ERROR).await?;
        return Ok(());
    }

    let mut db = ctx.data().t_db.lock().await;

    match db.rename_template(&from, &to) {
        Err(e) => {
            ctx.say(e.to_string()).await?;
        }
        Ok(result) => {
            if result {
                ctx.say(format!("Renamed template {} to {}.", &from, &to))
                    .await?;
            } else {
                ctx.say(format!("Error: No template named {} exists.", &from))
                    .await?;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn rename_substitute(
    ctx: Context<'_>,
    template: String,
    from: String,
    to: String,
) -> Result<(), Error> {
    let mut db = ctx.data().t_db.lock().await;

    match db.rename_substitute(&template, &from, &to) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say(format!("Error: No template named {} exists.", &template))
                    .await?;
            }
            _ => {
                ctx.say(e.to_string()).await?;
            }
        },
        Ok(result) => {
            if result {
                ctx.say(format!(
                    "Renamed substitute {} to {} in template {}.",
                    &from, &to, &template
                ))
                .await?;
            } else {
                ctx.say(format!(
                    "Error: No substitute exists in template {} named {}.",
                    &template, &from
                ))
                .await?;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: Context<'_>, template: Option<String>) -> Result<(), Error> {
    let db = ctx.data().t_db.lock().await;

    match template {
        Some(tmp) => match db.get_substitutes(&tmp) {
            Ok(subs) => {
                if subs.is_empty() {
                    ctx.say_ephemeral(&format!("No substitutes in template {}", tmp)[..])
                        .await?;
                } else {
                    ctx.say_vec(subs, true).await?;
                }
            }
            _ => {
                ctx.say_ephemeral("Error: Couldn't get subsitutes.").await?;
            }
        },
        None => match db.get_templates() {
            Ok(tmps) => {
                if tmps.is_empty() {
                    ctx.say_ephemeral(
                        "There are currently no templates. Try creaing some with /add",
                    )
                    .await?;
                } else {
                    ctx.say_vec(tmps, true).await?;
                }
            }
            _ => {
                ctx.say_ephemeral("Error: Couldn't get templates.").await?;
            }
        },
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn generate(ctx: Context<'_>, text: String) -> Result<(), Error> {
    let db = ctx.data().t_db.lock().await;
    let mut interpolator = TextInterpolator::default();

    let output = interpolator.interp(
        &text,
        &|template| match db.get_random_substitute(template) {
            Ok(sub) => Some(sub),
            Err(_) => None,
        },
    );

    match output {
        Ok(o) => {
            ctx.multi_say(&o).await?;
        }
        Err(e) => {
            ctx.say(format!("Error: {e}")).await?;
        }
    }

    Ok(())
}

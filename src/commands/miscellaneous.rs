use super::*;
use poise::serenity_prelude::{self as serenity};
use rand::Rng;

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}.", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn random_number(ctx: Context<'_>, min: String, max: String) -> Result<(), Error> {
    match rng(min, max) {
        Ok(result) => {
            ctx.say(result.to_string()).await?;
            Ok(())
        }
        Err(e) => {
            ctx.say(format!("Error: {}.", e)).await?;
            Ok(())
        }
    }
}

fn rng(min: String, max: String) -> Result<i64, &'static str> {
    match (min.parse(), max.parse()) {
        (Ok(min), Ok(max)) => {
            let mut rng = rand::thread_rng();
            if min < max {
                return Ok(rng.gen_range(min..=max));
            } else {
                return Err("minimum value must be less than maximum value");
            }
        }
        (Ok(_), Err(_)) => Err("failed to convert max value to a number"),
        (Err(_), Ok(_)) => Err("failed to convert min value to a number"),
        (Err(_), Err(_)) => Err("failed to convert min and max values to numbers"),
    }
}

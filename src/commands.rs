use poise::serenity_prelude::{self as serenity};
use rand::Rng;
use template_substitution_database::rusqlite;
use template_substitution_database::TemplateDatabase;
use text_interpolator::TextInterpolator;
use tokio::sync::Mutex;

pub struct Data {
    pub t_db: Mutex<TemplateDatabase>,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

const TEMPLATE_NAME_ERROR: &str = "Error: templates may only contain letters and numbers.";

struct OutputLog {
    present: String,
    not_present: String,
}

impl OutputLog {
    fn from(user_input: Vec<&str>, changes: Vec<&str>) -> Self {
        OutputLog {
            present: Self::stringify(&user_input, |input| changes.contains(&input)),
            not_present: Self::stringify(&user_input, |input| !changes.contains(&input)),
        }
    }

    fn stringify(strings: &Vec<&str>, predicate: impl Fn(&str) -> bool) -> String {
        let mut output: String = strings
            .iter()
            .filter(|input| predicate(input))
            .map(|s| {
                if s.contains(' ') {
                    format!("{}{}{}", "\"", s, "\" ")
                } else {
                    s.to_string() + " "
                }
            })
            .collect();
        output.pop();
        output
    }
}

#[derive(Debug)]
struct QuoteFilter<'a> {
    pub quoted: Vec<&'a str>,
    pub unquoted: Vec<&'a str>,
}

impl<'a> QuoteFilter<'a> {
    pub fn from(input: &'a str) -> Self {
        const EMPTY: (&str, &str) = ("", "");

        let mut quoted: Vec<&str> = Vec::new();
        let mut unquoted: Vec<&str> = Vec::new();

        let mut first_split = input.split_once("\"");
        let mut second_split = first_split.unwrap_or(EMPTY).1.split_once("\"");
        let mut left_overs = "";

        if first_split == None {
            left_overs = input;
        }

        while first_split != None && second_split != None {
            Self::push_if_not_empty(&mut unquoted, first_split.unwrap_or(EMPTY).0.trim());
            Self::push_if_not_empty(&mut quoted, second_split.unwrap_or(EMPTY).0.trim());
            first_split = (second_split).unwrap_or(EMPTY).1.split_once("\"");
            left_overs = second_split.unwrap_or(EMPTY).1;
            second_split = (first_split).unwrap_or(EMPTY).1.split_once("\"");
        }

        Self::push_if_not_empty(&mut unquoted, left_overs.trim());

        QuoteFilter { quoted, unquoted }
    }

    fn push_if_not_empty<'b>(input: &mut Vec<&'b str>, value: &'b str) {
        if !value.is_empty() {
            input.push(value);
        }
    }
}

fn vectorize_input(input: &str) -> Vec<&str> {
    let quote_filter = &QuoteFilter::from(&input);

    let mut output: Vec<&str> = Vec::new();

    for quoted in &quote_filter.quoted {
        output.push(&quoted);
    }

    for unquoted in &quote_filter.unquoted {
        for word in unquoted.split_whitespace() {
            output.push(word);
        }
    }

    output
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

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

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
        Some(tmp) => {
            if let Ok(subs) = db.get_substitutes(&tmp) {
                ctx.say(subs.join(", ")).await?;
            }
        }
        None => {
            if let Ok(tmps) = db.get_templates() {
                ctx.say(tmps.join(", ")).await?;
            }
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn generate(ctx: Context<'_>, text: String) -> Result<(), Error> {
    let db = ctx.data().t_db.lock().await;
    let mut interpolator = TextInterpolator::default();

    let output = interpolator.interp(&text, &|s| match db.get_substitutes(s) {
        Ok(subs) => {
            if subs.len() > 0 && !subs[0].is_empty() {
                let mut rng = rand::thread_rng();
                let i: usize = rng.gen_range(0..subs.len());
                Some(subs[i].clone())
            } else {
                None
            }
        }
        Err(_) => None,
    });

    match output {
        Ok(o) => {
            ctx.say(o).await?;
        }
        Err(e) => {
            ctx.say(format!("Error: {e}")).await?;
        }
    }

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

#[cfg(test)]
mod tests {
    use crate::commands::vectorize_input;

    #[test]
    fn mixed_quote_input() {
        let input = String::from(
            "cat \"\" \"United States of America\" bear snake lion \"my mom\"  \"ten bulls\" dog goat",
        );

        dbg!(&vectorize_input(&input));

        assert_eq!(vectorize_input(&input).len(), 9);
    }

    #[test]
    fn no_quote_input() {
        let input = String::from("This is some input");

        assert_eq!(vectorize_input(&input).len(), 4);
    }
}

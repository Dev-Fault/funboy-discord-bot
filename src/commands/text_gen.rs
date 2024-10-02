use crate::{
    interpreter::Interpreter,
    io_utils::{
        change_log::OutputLog,
        context_extension::{ContextExtension, MESSAGE_BYTE_LIMIT},
        discord_message_format::vectorize_input,
    },
    Context, Error,
};

use text_interpolator::TextInterpolator;

const INPUT_BYTE_LIMIT: usize = MESSAGE_BYTE_LIMIT;

const ERROR_INVALID_TEMPLATE_NAME: &str = "Error: templates may only contain letters and numbers.";
const ERROR_NO_TEMPLATES: &str =
    "Error: There are currently no templates. Try creaing some with /add";
const ERROR_DATABASE_QUERY: &str = "Error: There was a problem querying the database.";
const ERROR_GENERATION_FAILED: &str = "Error: Text generation failed.";
const ERROR_TEMPLATE_TOO_LARGE: &str = "Error: Template was too large.";
const ERROR_SUB_TOO_LARGE: &str = "Error: Substitute was too large.";

/// Adds multiple substitutes to a template.
///
/// To use this command type in a template (it doesn't have to exist yet)
/// then type in the substitutes that you want to add
///
/// Substitutes are seperated by spaces such as: **apple banana orange**
/// To add a multi-word substitute use quotes like: **"This is a multi-word substitute!"**
///
/// Example usage: **/add** template: **fruit** substitutes: **apple banana orange "dragon fruit" "key lime"**
#[poise::command(slash_command, prefix_command)]
pub async fn add(ctx: Context<'_>, template: String, substitutes: String) -> Result<(), Error> {
    if template.len() > INPUT_BYTE_LIMIT {
        ctx.say_ephemeral(ERROR_TEMPLATE_TOO_LARGE).await?;
        return Ok(());
    } else if template.contains(|c: char| !c.is_alphanumeric()) {
        ctx.say_ephemeral(ERROR_INVALID_TEMPLATE_NAME).await?;
        return Ok(());
    }

    let mut db = ctx.data().template_db.lock().await;

    let subs: Vec<&str> = vectorize_input(substitutes.as_str());

    for sub in &subs {
        if sub.len() > INPUT_BYTE_LIMIT {
            ctx.say_ephemeral(ERROR_SUB_TOO_LARGE).await?;
            return Ok(());
        }
    }

    match db.insert_subs(&template, Some(&subs)) {
        Err(e) => {
            ctx.say_ephemeral(&e.to_string()).await?;
        }
        Ok(inserted_subs) => {
            let output_log = OutputLog::from(subs, inserted_subs);

            ctx.multi_say(
                &format!(
                    "Added substitutes [{}] under template **\"**{}**\"**.",
                    &output_log.present, &template
                )[..],
                false,
            )
            .await?;

            if output_log.not_present.len() > 0 {
                ctx.multi_say(
                    &format!(
                        "Substitutes [{}] are already present under template **\"**{}**\"**.",
                        &output_log.not_present, &template
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Adds a single substitute to a template.
///
/// To use this command type in a template (it doesn't have to exist yet)
/// then type in the substitute that you want to add
///
/// The substitute can be a single word or multiple words like:
/// **apple** or **I want to eat an apple**
/// both will be added as a single substitute under the chosen template.
///
/// Example usage: **/add_sub** template: **fruit** substitute: **I love apples!**
#[poise::command(slash_command, prefix_command)]
pub async fn add_sub(ctx: Context<'_>, template: String, substitute: String) -> Result<(), Error> {
    if template.contains(|c: char| !c.is_alphanumeric()) {
        ctx.say_ephemeral(ERROR_INVALID_TEMPLATE_NAME).await?;
        return Ok(());
    } else if template.len() > INPUT_BYTE_LIMIT {
        ctx.say_ephemeral(ERROR_TEMPLATE_TOO_LARGE).await?;
        return Ok(());
    } else if substitute.len() > INPUT_BYTE_LIMIT {
        ctx.say_ephemeral(ERROR_SUB_TOO_LARGE).await?;
        return Ok(());
    }

    let mut db = ctx.data().template_db.lock().await;

    match db.insert_sub(&template, &substitute) {
        Err(e) => {
            ctx.say_ephemeral(&e.to_string()).await?;
        }
        Ok(result) => {
            if result {
                ctx.multi_say(
                    &format!(
                        "Added substitute **\"**{}**\"** under template **\"**{}**\"**.",
                        &substitute, &template
                    )[..],
                    false,
                )
                .await?;
            } else {
                ctx.multi_say(
                    &format!(
                        "Substitute **\"**{}**\"** is already present under template **\"**{}**\"**.",
                        &substitute, &template
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Removes a template.
///
/// **Warning:** This command will permanently delete a template and all of it's substitutes
/// this action cannot be undone!
///
/// To use this command type in the template you want to remove
///
/// Example usage: **/remove_template** template: **fruit**
#[poise::command(slash_command, prefix_command)]
pub async fn remove_template(ctx: Context<'_>, template: String) -> Result<(), Error> {
    let mut db = ctx.data().template_db.lock().await;

    match db.remove_template(&template) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say_ephemeral(&format!(
                    "No template named **\"**{}**\"** exists.",
                    &template
                ))
                .await?;
            }
            _ => {
                ctx.say_ephemeral(&e.to_string()).await?;
            }
        },
        Ok(result) => {
            if result {
                ctx.say(format!("Removed template **\"**{}**\"**.", &template))
                    .await?;
            } else {
                ctx.say_ephemeral(&format!(
                    "No template named **\"**{}**\"** exists.",
                    &template
                ))
                .await?;
            }
        }
    }

    Ok(())
}

/// Removes a single substitute from a template.
///
/// To use this command type in the name of a template
/// and then the substitute within the template that you want to delete
///
/// Example usage: **/remove_sub** template: **fruit** substitute: **I love apples!**
#[poise::command(slash_command, prefix_command)]
pub async fn remove_sub(
    ctx: Context<'_>,
    template: String,
    substitute: String,
) -> Result<(), Error> {
    let mut db = ctx.data().template_db.lock().await;

    match db.remove_sub(&template, &substitute) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say_ephemeral(&format!(
                    "No template named **\"**{}**\"** exists.",
                    &template
                ))
                .await?;
            }
            _ => {
                ctx.say_ephemeral(&e.to_string()).await?;
            }
        },
        Ok(result) => {
            if result {
                ctx.multi_say(
                    &format!(
                        "Removed substitute **\"**{}**\"** from template **\"**{}**\"**.",
                        &substitute, &template
                    )[..],
                    false,
                )
                .await?;
            } else {
                ctx.multi_say(
                    &format!(
                        "Substitute **\"**{}**\"** was not found in template **\"**{}**\"**.",
                        &substitute, &template
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Removes multiple substitutes from a template.
///
/// To use this command type in the name of a template
/// and then the substitutes within the template that you want to delete
///
/// Example usage: **/remove_sub** template: **fruit** substitute: **"dragon fruit" apple banana "I love apples!"**
#[poise::command(slash_command, prefix_command)]
pub async fn remove_subs(
    ctx: Context<'_>,
    template: String,
    substitutes: String,
) -> Result<(), Error> {
    let mut db = ctx.data().template_db.lock().await;

    let subs_to_remove: Vec<&str> = vectorize_input(substitutes.as_str());

    match db.remove_subs(&template, &subs_to_remove) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say_ephemeral(&format!(
                    "No template named **\"**{}**\"** exists.",
                    &template
                ))
                .await?;
            }
            _ => {
                ctx.say_ephemeral(&e.to_string()).await?;
            }
        },
        Ok(removed_subs) => {
            let output_log = OutputLog::from(subs_to_remove, removed_subs);
            ctx.multi_say(
                &format!(
                    "Removed substitutes [{}] from template **\"**{}**\"**.",
                    output_log.present, &template
                )[..],
                false,
            )
            .await?;
            if output_log.not_present.len() > 0 {
                ctx.multi_say(
                    &format!(
                        "Substitutes [{}] were not found in template **\"**{}**\"**.",
                        output_log.not_present, &template
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Replaces an existing substitute with a new substitute.
///
/// To use this command type in an existing template
/// then type in an existing substitute followed by the replacement substitute
///
/// Example usage: **/replace_sub** template: **fruit** old_sub: **apple** new_sub: **orange**
#[poise::command(slash_command, prefix_command)]
pub async fn replace_sub(
    ctx: Context<'_>,
    template: String,
    old_sub: String,
    new_sub: String,
) -> Result<(), Error> {
    let mut db = ctx.data().template_db.lock().await;

    if new_sub.len() > INPUT_BYTE_LIMIT {
        ctx.say_ephemeral(ERROR_SUB_TOO_LARGE).await?;
        return Ok(());
    }

    match db.rename_substitute(&template, &old_sub, &new_sub) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say_ephemeral(&format!(
                    "No template named **\"**{}**\"** exists.",
                    &template
                ))
                .await?;
            }
            _ => {
                ctx.say_ephemeral(&e.to_string()).await?;
            }
        },
        Ok(result) => {
            if result {
                ctx.multi_say(
                    &format!(
                        "Renamed substitute **\"**{}**\"** to **\"**{}**\"** in template **\"**{}**\"**.",
                        &old_sub, &new_sub, &template
                    )[..],
                    false,
                )
                .await?;
            } else {
                ctx.multi_say(
                    &format!(
                        "No substitute exists in template **\"**{}**\"** named **\"**{}**\"**.",
                        &template, &old_sub
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Renames a template.
///
/// To use this command type in the template you want to rename followed by the new template name
///
/// Example usage: **/rename_template** from: **fruit** to: **vegtable**
#[poise::command(slash_command, prefix_command)]
pub async fn rename_template(ctx: Context<'_>, from: String, to: String) -> Result<(), Error> {
    if from.contains(|c: char| !c.is_alphanumeric()) || to.contains(|c: char| !c.is_alphanumeric())
    {
        ctx.say_ephemeral(ERROR_INVALID_TEMPLATE_NAME).await?;
        return Ok(());
    } else if to.len() > INPUT_BYTE_LIMIT {
        ctx.say_ephemeral(ERROR_TEMPLATE_TOO_LARGE).await?;
        return Ok(());
    }

    let mut db = ctx.data().template_db.lock().await;

    match db.rename_template(&from, &to) {
        Err(e) => {
            ctx.say_ephemeral(&e.to_string()).await?;
        }
        Ok(result) => {
            if result {
                ctx.say(format!(
                    "Renamed template **\"**{}**\"** to **\"**{}**\"**.",
                    &from, &to
                ))
                .await?;
            } else {
                ctx.say_ephemeral(&format!("No template named **\"**{}**\"** exists.", &from))
                    .await?;
            }
        }
    }

    Ok(())
}

/// Lists all templates or optionally substitutes inside a template.
///
/// To list all templates simply type /list and press enter
/// To list all substitutes within a template type /list and then the name of a template
///
/// Example usage: **/list** template: **fruit**
#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: Context<'_>, template: Option<String>) -> Result<(), Error> {
    let db = ctx.data().template_db.lock().await;

    match template {
        Some(tmp) => match db.get_subs(&tmp) {
            Ok(subs) => {
                if subs.is_empty() {
                    ctx.say_ephemeral(
                        &format!("No substitutes in template **\"**{}**\"**", tmp)[..],
                    )
                    .await?;
                } else {
                    ctx.say_vec(subs, true).await?;
                }
            }
            _ => {
                ctx.say_ephemeral(
                    &format!(
                        "Error: Couldn't get any subsitutes for template **\"**{}**\"**.",
                        tmp
                    )[..],
                )
                .await?;
            }
        },
        None => match db.get_templates() {
            Ok(tmps) => {
                if tmps.is_empty() {
                    ctx.say_ephemeral(ERROR_NO_TEMPLATES).await?;
                } else {
                    ctx.say_vec(tmps, true).await?;
                }
            }
            _ => {
                ctx.say_ephemeral(ERROR_DATABASE_QUERY).await?;
            }
        },
    }
    Ok(())
}

/// Generates randomized text by replacing templates with a random substitute.
///
/// To use this command enter text optionally containing templates headed with any of the following
/// characters ' ^ `
/// Example: **I love 'fruit**
/// Example 2: **I love ^fruit**
///
/// You can add a suffix to a template by including another template character at the end of the
/// template name:
/// **^verb^ing** which may be subsituted into **flying** if a substitute exists named **fly**
///
/// Example usage: **/generate I love ^fruit^s**
/// Example output: **I love apples!**
#[poise::command(slash_command, prefix_command)]
pub async fn generate(ctx: Context<'_>, text: String) -> Result<(), Error> {
    let db = ctx.data().template_db.lock().await;
    let mut interpolator = TextInterpolator::default();

    let output = interpolator.interp(&text, &|template| match db.get_random_subs(template) {
        Ok(sub) => Some(sub),
        Err(_) => None,
    });

    match output {
        Ok(output) => {
            match interpret_code(&output) {
                Ok(o) => ctx.multi_say(&o, false).await?,
                Err(e) => ctx.say_ephemeral(&format!("Error: {}.", &e)[..]).await?,
            };
        }
        Err(_) => {
            ctx.say(ERROR_GENERATION_FAILED).await?;
        }
    }

    Ok(())
}

fn interpret_code(input: &str) -> Result<String, String> {
    let mut output = String::with_capacity(input.len());
    let mut code_stack: Vec<String> = Vec::new();
    let mut interpreter = Interpreter::new();

    let mut code_depth: i16 = 0;

    for c in input.chars() {
        if c == '{' {
            code_stack.push(String::new());
            code_depth += 1;
        } else if c == '}' {
            code_depth -= 1;
            if code_depth < 0 {
                return Err("Unmatched curly braces".to_string());
            } else {
                match code_stack.pop() {
                    Some(code) => {
                        match interpreter.interpret(&code) {
                            Ok(eval) => match code_stack.last_mut() {
                                Some(code) => code.push_str(&eval),
                                None => output.push_str(&eval),
                            },
                            Err(e) => return Err(e),
                        };
                    }
                    None => {}
                }
            }
        } else if code_depth == 0 {
            output.push(c);
        } else {
            match code_stack.last_mut() {
                Some(s) => s.push(c),
                None => {}
            }
        }
    }

    if code_depth != 0 {
        return Err("Unmatched curly braces".to_string());
    }

    Ok(output)
}

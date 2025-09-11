use core::panic;

use crate::io_utils::context_extension::MessageListFormatter;
use crate::io_utils::custom_components::{
    create_confirmation_interaction, CANCEL_BUTTON_ID, CONFIRM_BUTTON_ID,
};
use crate::io_utils::discord_message_format;
use crate::io_utils::input_interp::interp_input;
use crate::{
    io_utils::{
        change_log::OutputLog,
        context_extension::{ContextExtension, MESSAGE_BYTE_LIMIT},
        discord_message_format::vectorize_input,
    },
    Context, Error,
};

const INPUT_BYTE_LIMIT: usize = MESSAGE_BYTE_LIMIT;

const ERROR_INVALID_TEMPLATE_NAME: &str = "Error: templates may only contain letters and numbers.";
const ERROR_NO_TEMPLATES: &str =
    "Error: There are currently no templates. Try creaing some with /add";
const ERROR_DATABASE_QUERY: &str = "Error: There was a problem querying the database.";
const ERROR_GENERATION_FAILED: &str = "Error: Text generation failed.";
const ERROR_TEMPLATE_TOO_LARGE: &str = "Error: Template was too large.";
const ERROR_SUB_TOO_LARGE: &str = "Error: Substitute was too large.";
const REMOVE_TEMPLATE_WARNING: &str =
    "Are you sure you want to remove this template? All of it's substitutes will be deleted.";

/// Add and create templates with multiple substitutes
///
/// A template is an alias that refers to one or more substitutes
///
/// A template must not have any special characters but otherwise can be anything
///
/// **Tip:** Add a single word substitutes by seperating them with spaces: **cat dog car house**
/// **Tip:** a multi-word substitute by surround it in quotes: **"Big brown dog"**
///
/// Example usage: **/add_subs** template: **fruit** substitutes: **apple banana orange "dragon fruit" "key lime"**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn add_subs(
    ctx: Context<'_>,
    template: String,
    substitutes: String,
) -> Result<(), Error> {
    if template.len() > INPUT_BYTE_LIMIT {
        ctx.say_ephemeral(ERROR_TEMPLATE_TOO_LARGE).await?;
        return Ok(());
    } else if template.contains(|c: char| !c.is_alphanumeric()) {
        ctx.say_ephemeral(ERROR_INVALID_TEMPLATE_NAME).await?;
        return Ok(());
    }

    let subs: Vec<&str> = vectorize_input(substitutes.as_str());

    for sub in &subs {
        if sub.len() > INPUT_BYTE_LIMIT {
            ctx.say_ephemeral(ERROR_SUB_TOO_LARGE).await?;
            return Ok(());
        }
    }

    let mut db = ctx.data().funboy_db.lock().await;
    match db.insert_subs(&template, Some(&subs)) {
        Err(e) => {
            ctx.say_ephemeral(&e.to_string()).await?;
        }
        Ok(inserted_subs) => {
            let output_log = OutputLog::from(subs, inserted_subs);

            ctx.say_long(
                &format!(
                    "Added substitutes [{}] to template **\"**{}**\"**.",
                    &output_log.present, &template
                )[..],
                false,
            )
            .await?;

            if output_log.not_present.len() > 0 {
                ctx.say_long(
                    &format!(
                        "Substitutes [{}] were already present in template **\"**{}**\"**.",
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

/// Add a single substitute to a template
///
/// Example usage: **/add_sub** template: **quote** substitute: **Quoth the raven, "Nevermore."**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
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

    let mut db = ctx.data().funboy_db.lock().await;

    match db.insert_sub(&template, &substitute) {
        Err(e) => {
            ctx.say_ephemeral(&e.to_string()).await?;
        }
        Ok(result) => {
            if result {
                ctx.say_long(
                    &format!(
                        "Added substitute **\"**{}**\"** to template **\"**{}**\"**.",
                        &substitute, &template
                    )[..],
                    false,
                )
                .await?;
            } else {
                ctx.say_long(
                    &format!(
                        "Substitute **\"**{}**\"** is already present in template **\"**{}**\"**.",
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

/// Copy substitutes from one template into another one
///
/// Example usage: **/copy_subs** from_template: **fruit** to_template: **produce**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn copy_subs(
    ctx: Context<'_>,
    from_template: String,
    to_template: String,
) -> Result<(), Error> {
    let mut db = ctx.data().funboy_db.lock().await;

    if let Ok(subs) = db.get_subs(&from_template) {
        let subs: Vec<&str> = subs.iter().map(|s| s.as_str()).collect();

        match db.insert_subs(&to_template, Some(&subs)) {
            Err(e) => {
                ctx.say_ephemeral(&e.to_string()).await?;
            }
            Ok(inserted_subs) => {
                let output_log = OutputLog::from(subs, inserted_subs);

                ctx.say_long(
                    &format!(
                        "Added substitutes [{}] to template **\"**{}**\"**.",
                        &output_log.present, &to_template
                    )[..],
                    false,
                )
                .await?;

                if output_log.not_present.len() > 0 {
                    ctx.say_long(
                        &format!(
                            "Substitutes [{}] were already present in template **\"**{}**\"**.",
                            &output_log.not_present, &to_template
                        )[..],
                        true,
                    )
                    .await?;
                }
            }
        }
    } else {
        ctx.say_ephemeral(&format!(
            "Error: Couldn't get any subs from template **\"**{}**\"**.",
            &from_template
        ))
        .await?;
    }

    Ok(())
}

/// Remove a template
///
/// **Warning:** This command will permanently delete a template and all of it's substitutes
/// this action cannot be undone!
///
/// Example usage: **/remove_template** template: **fruit**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn remove_template(ctx: Context<'_>, template: String) -> Result<(), Error> {
    match create_confirmation_interaction(ctx, REMOVE_TEMPLATE_WARNING, 30).await? {
        Some(interaction) => match interaction.data.custom_id.as_str() {
            CANCEL_BUTTON_ID => {
                interaction
                    .create_response(
                        ctx.http(),
                        serenity::all::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;

                ctx.say_ephemeral("Command to remove template canceled.")
                    .await?
            }
            CONFIRM_BUTTON_ID => {
                interaction
                    .create_response(
                        ctx.http(),
                        serenity::all::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;

                let mut db = ctx.data().funboy_db.lock().await;
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
            }
            _ => {
                panic!("Incorrect id for remove template confirmation interaction.")
            }
        },
        None => {
            ctx.say_ephemeral("Timeout: Command to remove template canceled.")
                .await?;
        }
    }

    Ok(())
}

/// Remove a single substitute from a template
///
/// Example usage: **/remove_sub** template: **fruit** substitute: **I love apples!**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn remove_sub(
    ctx: Context<'_>,
    template: String,
    substitute: String,
) -> Result<(), Error> {
    let mut db = ctx.data().funboy_db.lock().await;

    match db.remove_sub(&template, &substitute) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say_ephemeral(&format!(
                    "No template named **\"**{}**\"** exists.",
                    template
                ))
                .await?;
            }
            _ => {
                ctx.say_ephemeral(&e.to_string()).await?;
            }
        },
        Ok(result) => {
            if result {
                ctx.say_long(
                    &format!(
                        "Removed substitute **\"**{}**\"** from template **\"**{}**\"**.",
                        substitute, template
                    )[..],
                    false,
                )
                .await?;
            } else {
                ctx.say_long(
                    &format!(
                        "Substitute **\"**{}**\"** was not found in template **\"**{}**\"**.",
                        substitute, template
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }
    Ok(())
}

/// Remove a single substitute with it's id from a template
///
/// **Tip:** To get the id of a substitute use the command /list_ids
///
/// Example usage: **/remove_sub** template: **fruit** id: **1234**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn remove_sub_by_id(ctx: Context<'_>, template: String, id: usize) -> Result<(), Error> {
    let mut db = ctx.data().funboy_db.lock().await;

    match db.remove_sub_by_id(&template, id) {
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ctx.say_ephemeral(&format!(
                    "No template named **\"**{}**\"** exists.",
                    template
                ))
                .await?;
            }
            _ => {
                ctx.say_ephemeral(&e.to_string()).await?;
            }
        },
        Ok(result) => {
            if result {
                ctx.say_long(
                    &format!(
                        "Removed substitute with id **{}** from template **\"**{}**\"**.",
                        id, template
                    )[..],
                    false,
                )
                .await?;
            } else {
                ctx.say_long(
                    &format!(
                        "Substitute with id **{}** was not found in template **\"**{}**\"**.",
                        id, template
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }
    Ok(())
}

/// Remove multiple substitutes from a template
///
/// **Tip:** Remove a single word substitutes by seperating them with spaces: **cat dog car house**
/// **Tip:** Remove a multi-word substitute by surround it in quotes: **"Big brown dog"**
///
/// Example usage: **/remove_subs** template: **fruit** substitutes: **"dragon fruit" apple banana "I love apples!"**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn remove_subs(
    ctx: Context<'_>,
    template: String,
    substitutes: String,
) -> Result<(), Error> {
    let subs_to_remove: Vec<&str> = vectorize_input(substitutes.as_str());

    let mut db = ctx.data().funboy_db.lock().await;
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
            ctx.say_long(
                &format!(
                    "Removed substitutes [{}] from template **\"**{}**\"**.",
                    output_log.present, &template
                )[..],
                false,
            )
            .await?;
            if output_log.not_present.len() > 0 {
                ctx.say_long(
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

/// Remove multiple substitutes with their ids from a template
///
/// **Tip:** Seperate each id by a space: 1234 4321
///
/// Example usage: **/remove_subs_by_id** template: **noun** ids: 0 1 2 3 4
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn remove_subs_by_id(
    ctx: Context<'_>,
    template: String,
    ids: String,
) -> Result<(), Error> {
    let mut invalid_ids: Vec<&str> = Vec::new();
    let subs_to_remove: Vec<usize> = vectorize_input(ids.as_str())
        .iter()
        .filter_map(|id| match id.parse::<usize>() {
            Ok(value) => Some(value),
            Err(_) => {
                invalid_ids.push(id);
                None
            }
        })
        .collect();

    if !invalid_ids.is_empty() {
        ctx.say_ephemeral(&format!("Ignoring invalid ids {:?}", invalid_ids))
            .await?;
    }

    let mut db = ctx.data().funboy_db.lock().await;
    match db.remove_subs_by_id(&template, &subs_to_remove) {
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
        Ok(removed_ids) => {
            let ephemeral;
            let message = match removed_ids.len() {
                0 => {
                    ephemeral = true;
                    "No substitutes were removed.".to_string()
                }
                _ => {
                    ephemeral = false;
                    format!(
                        "Removed substitutes with ids {:?} from template **\"**{}**\"**.",
                        removed_ids, &template
                    )
                }
            };

            ctx.say_long(&message[..], ephemeral).await?;

            let ignored_ids: Vec<usize> = subs_to_remove
                .iter()
                .filter(|id| !removed_ids.contains(id))
                .map(|id| *id)
                .collect();
            if ignored_ids.len() > 0 {
                ctx.say_long(
                    &format!(
                        "Substitutes with ids {:?} were not found in template **\"**{}**\"**.",
                        ignored_ids, &template
                    )[..],
                    true,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Replace an old substitute with a new one
///
/// Example usage: **/replace_sub** template: **fruit** old_sub: **apple** new_sub: **orange**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn replace_sub(
    ctx: Context<'_>,
    template: String,
    old_sub: String,
    new_sub: String,
) -> Result<(), Error> {
    {
        if new_sub.len() > INPUT_BYTE_LIMIT {
            ctx.say_ephemeral(ERROR_SUB_TOO_LARGE).await?;
            return Ok(());
        }

        let mut db = ctx.data().funboy_db.lock().await;
        match db.replace_substitute(&template, &old_sub, &new_sub) {
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ctx.say_ephemeral(&format!(
                        "No template named **\"**{}**\"** exists.",
                        template
                    ))
                    .await?;
                }
                _ => {
                    ctx.say_ephemeral(&e.to_string()).await?;
                }
            },
            Ok(result) => {
                if result {
                    ctx.say_long(
                            &format!(
                                "Renamed substitute **\"**{}**\"** to **\"**{}**\"** in template **\"**{}**\"**.",
                                old_sub, new_sub, template
                            )[..],
                            false,
                        )
                        .await?;
                } else {
                    ctx.say_long(
                        &format!(
                            "No substitute exists in template **\"**{}**\"** named **\"**{}**\"**.",
                            template, old_sub
                        )[..],
                        true,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

/// Replace an old substitute with it's id with a new one
///
/// Example usage: **/replace_sub_by_id** template: **fruit** id: **1234** new_sub: **orange**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn replace_sub_by_id(
    ctx: Context<'_>,
    template: String,
    id: usize,
    new_sub: String,
) -> Result<(), Error> {
    {
        if new_sub.len() > INPUT_BYTE_LIMIT {
            ctx.say_ephemeral(ERROR_SUB_TOO_LARGE).await?;
            return Ok(());
        }

        let mut db = ctx.data().funboy_db.lock().await;
        match db.replace_substitute_by_id(&template, id, &new_sub) {
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ctx.say_ephemeral(&format!(
                        "No template named **\"**{}**\"** exists.",
                        template
                    ))
                    .await?;
                }
                _ => {
                    ctx.say_ephemeral(&e.to_string()).await?;
                }
            },
            Ok(result) => {
                if result {
                    ctx.say_long(
                            &format!(
                                "Renamed substitute with id **{}** to **\"**{}**\"** in template **\"**{}**\"**.",
                                id, new_sub, template
                            )[..],
                            false,
                        )
                        .await?;
                } else {
                    ctx.say_long(
                        &format!(
                            "No substitute exists in template **\"**{}**\"** with id **{}**.",
                            template, id
                        )[..],
                        true,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

/// Rename a template
///
/// **Tip:** If this template is referenced inside of another template it will also rename
/// the refernce
///
/// Example usage: **/rename_template** from: **fruit** to: **vegtable**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn rename_template(ctx: Context<'_>, from: String, to: String) -> Result<(), Error> {
    if from.contains(|c: char| !c.is_alphanumeric()) || to.contains(|c: char| !c.is_alphanumeric())
    {
        ctx.say_ephemeral(ERROR_INVALID_TEMPLATE_NAME).await?;
        return Ok(());
    } else if to.len() > INPUT_BYTE_LIMIT {
        ctx.say_ephemeral(ERROR_TEMPLATE_TOO_LARGE).await?;
        return Ok(());
    }

    let mut db = ctx.data().funboy_db.lock().await;

    match db.rename_template(&from, &to) {
        Err(e) => {
            eprintln!("Error: {}", e);
            ctx.say_ephemeral(&format!("Error: Failed to rename template. Make sure no preexisting templates exist named **{}**.", to))
                .await?;
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

async fn say_list(
    ctx: Context<'_>,
    template: Option<String>,
    formatter: MessageListFormatter,
    show_ids: bool,
) -> Result<(), Error> {
    let db = ctx.data().funboy_db.lock().await;

    match template {
        Some(tmp) => match db.get_sub_records(&tmp) {
            Ok(subs) => {
                if subs.is_empty() {
                    ctx.say_ephemeral(
                        &format!("No substitutes in template **\"**{}**\"**", tmp)[..],
                    )
                    .await?;
                } else if show_ids {
                    let subs_with_ids: Vec<String> = subs
                        .iter()
                        .map(|record| {
                            format!(
                                "**ID:** {}\n**Substitute:**\n{}\n\n",
                                record.id, record.name
                            )
                        })
                        .collect();

                    ctx.say_list(
                        &subs_with_ids
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>()[..],
                        true,
                        None,
                    )
                    .await?;
                } else {
                    ctx.say_list(
                        &subs
                            .iter()
                            .map(|record| record.name.as_str())
                            .collect::<Vec<&str>>()[..],
                        true,
                        Some(formatter),
                    )
                    .await?;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", &e.to_string());

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
                    ctx.say_list(
                        &tmps.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[..],
                        true,
                        Some(formatter),
                    )
                    .await?;
                }
            }
            _ => {
                ctx.say_ephemeral(ERROR_DATABASE_QUERY).await?;
            }
        },
    }
    Ok(())
}

/// List all templates or substitutes in a template
///
/// Example usage: **/list** template: **fruit**
/// Example usage: **/list**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn list(ctx: Context<'_>, template: Option<String>) -> Result<(), Error> {
    say_list(
        ctx,
        template,
        discord_message_format::format_as_standard_list,
        false,
    )
    .await?;
    Ok(())
}

/// List substitues in a template with their ids
///
/// Example usage: **/list_ids** template: **noun**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn list_ids(ctx: Context<'_>, template: String) -> Result<(), Error> {
    say_list(
        ctx,
        Some(template),
        discord_message_format::format_as_standard_list,
        true,
    )
    .await?;
    Ok(())
}

/// List substitutes or templates numerically ordered
///
/// Example usage: **/list_numerically** template: **noun**
/// Example usage: **/list_numerically**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn list_numerically(ctx: Context<'_>, template: Option<String>) -> Result<(), Error> {
    say_list(
        ctx,
        template,
        discord_message_format::format_as_numeric_list,
        false,
    )
    .await?;
    Ok(())
}

/// Generate randomized text by replacing templates with a random substitute.
///
/// To use this command enter text optionally containing templates headed with any of the following
/// characters ' ^ `. If text is headed by one of these characters the generator will attempt to
/// replace it with a random substitute.
/// Example: **I love 'fruit**
/// Possible output: **I love apple**
/// Example 2: **I love ^fruit**
/// Possible output: **I love banana**
///
/// You can add a suffix to a template by including another template character at the end of the
/// template name:
/// **^verb^ing** which may be subsituted into **eating** if a substitute exists named **eat**
///
/// You can also embed code to manipulate text using the FSL language. Type **/fsl_help** for more
/// information.
///
/// Example usage: **/generate I love ^fruit^s**
/// Example output: **I love apples!**
#[poise::command(slash_command, prefix_command, category = "Text substitution")]
pub async fn generate(ctx: Context<'_>, text: String) -> Result<(), Error> {
    let interpreted_prompt = interp_input(&text, ctx.data().funboy_db.clone()).await;

    match interpreted_prompt {
        Ok(output) => ctx.say_long(&output, false).await?,
        Err(e) => {
            ctx.say_ephemeral(&format!("Error: {}", &e)).await?;
        }
    };

    Ok(())
}

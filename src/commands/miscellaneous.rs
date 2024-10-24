use crate::{
    fsl_documentation::get_command_documentation,
    io_utils::{context_extension::ContextExtension, discord_message_format::extract_image_urls},
    Context, Error,
};

use poise::{
    samples::HelpConfiguration,
    serenity_prelude::{self as serenity, ChannelId, CreateEmbed, CreateMessage},
    CreateReply,
};

/// Display help information for commands
#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let bottom_text = "\
    Type \"/help name_of_command\" for more info on a specific command.";

    let config = HelpConfiguration {
        show_subcommands: false,
        show_context_menu_commands: false,
        ephemeral: true,
        extra_text_at_bottom: bottom_text,
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Display help information for commands
#[poise::command(slash_command, prefix_command)]
pub async fn fsl_help(ctx: Context<'_>, command_name: Option<String>) -> Result<(), Error> {
    match command_name {
        Some(command_name) => {
            if let Some(command) = get_command_documentation()
                .iter()
                .find(|info| info.name == command_name)
            {
                let mut examples = command
                    .examples
                    .iter()
                    .map(|example| "**".to_string() + example + "**\n")
                    .collect::<String>();

                examples.pop();

                let embed = CreateEmbed::new()
                    .title(command.name.clone())
                    .description(format!(
                        "Description: {}\n\nExamples:\n{}",
                        command.description, examples
                    ));

                ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                    .await?;
            } else {
                ctx.say_ephemeral(&format!("No command named **{}** was found.", command_name))
                    .await?;
            }
        }
        None => {
            let title = "FSL - Funboy Scripting Language";
            let description = "The FSL language is a simple scripting language that you can embed in text when using the /generate command to manipulate text.\n
        When using the **/generate** command to indicate you are using FSL commands place any code inside of curly braces such as in this example: **{print(\"Hello, world!\")}**\n
        The generate command will then interpret the code inside the curly braces and replace the curly braces and text within with the generated output.\n
        As an example: **/generate Hello {print(\", world!\")}** will output: **Hello, World!**\n
        The FSL language recognizes a few different types of data: Int (whole number), Float (decimal number), Text (indicated by surrounding data in quotes), Identifier (text that is not surrounded by quotes), and List which are an aggregate of any of the preceeding types and can be created using the Copy command.\n
        To use the language familiarize yourself with the commands by typing **/help_fsl command_name** to get more information on a specific command.";

            let mut list_of_commands: String = get_command_documentation()
                .iter()
                .map(|info| info.name.to_string() + ", ")
                .collect::<String>();

            list_of_commands.pop();
            list_of_commands.pop();

            let embed = CreateEmbed::new()
                .title(title)
                .description(format!("{}\n\nCommands: {}", description, list_of_commands));

            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Move pinned messages posted by the bot to a selected channel
///
/// Example usage: **/move_bot_pins** to_channel: **my-channel**
#[poise::command(slash_command, prefix_command)]
pub async fn move_bot_pins(ctx: Context<'_>, to_channel: String) -> Result<(), Error> {
    if let Some(to_id) = get_channel_id(ctx, &to_channel).await? {
        let pins = ctx.channel_id().pins(ctx.http()).await?;
        for pin in pins {
            let bot_user = ctx.http().get_current_user().await?;
            if &pin.author.name == &bot_user.name {
                let mut embed = CreateEmbed::new()
                    .title(&pin.author.name)
                    .description(&pin.content)
                    .url(pin.link());

                let image_urls = extract_image_urls(&pin.content);

                if image_urls.len() == 1 {
                    embed = embed.image(image_urls[0]);
                    ctx.defer().await?;
                    to_id
                        .send_message(&ctx.http(), CreateMessage::new().embed(embed))
                        .await?;
                } else {
                    ctx.defer().await?;
                    to_id
                        .send_message(&ctx.http(), CreateMessage::new().embed(embed))
                        .await?;

                    for image_url in extract_image_urls(&pin.content) {
                        ctx.defer().await?;
                        to_id
                            .send_message(
                                &ctx.http(),
                                CreateMessage::new().embed(CreateEmbed::new().image(image_url)),
                            )
                            .await?;
                    }
                }

                pin.unpin(ctx.http()).await?;
            }
        }
        ctx.defer().await?;
        ctx.send(CreateReply::default().content(format!(
            "Succesfully moved pins to channel **{}**.",
            to_channel
        )))
        .await?;
    } else {
        ctx.say(format!(
            "Error: Could not find channel with name **{}**.",
            to_channel
        ))
        .await?;
    }
    Ok(())
}

async fn get_channel_id(ctx: Context<'_>, channel_name: &str) -> Result<Option<ChannelId>, Error> {
    match ctx.guild_id() {
        Some(guild_id) => {
            let guild = guild_id.to_partial_guild(ctx).await?;
            for (channel_id, channel) in guild.channels(ctx).await?.iter() {
                if channel.name() == channel_name {
                    return Ok(Some(*channel_id));
                }
            }
            return Ok(None);
        }
        None => Ok(None),
    }
}

/// Display the age of a users account.
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

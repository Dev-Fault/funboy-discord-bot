use crate::{io_utils::discord_message_format::extract_image_urls, Context, Error};

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

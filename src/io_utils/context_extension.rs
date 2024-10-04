use crate::{Context, Error};

use poise::CreateReply;

use super::discord_message_format::{split_long_string, split_message, DISCORD_CHARACTER_LIMIT};

pub const MESSAGE_BYTE_LIMIT: usize = DISCORD_CHARACTER_LIMIT * 4;
pub const WARN_MESSAGE_SIZE_EXCEEDED: &str = "Message was too large to send.";
pub const WARN_EMPTY_MESSAGE: &str = "Message was empty.";

pub type MessageListFormatter = fn(&[&str]) -> Vec<String>;

pub trait ContextExtension {
    async fn say_list(
        &self,
        message: &[&str],
        ephemeral: bool,
        formatter: Option<MessageListFormatter>,
    ) -> Result<(), Error>;

    async fn say_ephemeral(&self, message: &str) -> Result<(), Error>;

    async fn say_long(&self, message: &str, ephemeral: bool) -> Result<(), Error>;
}

impl<'a> ContextExtension for Context<'a> {
    async fn say_list(
        &self,
        message: &[&str],
        ephemeral: bool,
        formatter: Option<MessageListFormatter>,
    ) -> Result<(), Error> {
        let mut size: usize = 0;

        for string in &message[..] {
            size = size.saturating_add(string.len());
        }

        if !ephemeral && size > MESSAGE_BYTE_LIMIT {
            self.say_ephemeral(WARN_MESSAGE_SIZE_EXCEEDED).await?;
            return Ok(());
        } else if size == 0 {
            self.say_ephemeral(WARN_EMPTY_MESSAGE).await?;
            return Ok(());
        }

        let formatted_message: Vec<String>;
        let message = match formatter {
            Some(formatter) => {
                formatted_message = formatter(message);
                &formatted_message
                    .iter()
                    .map(|msg| msg.as_str())
                    .collect::<Vec<&str>>()[..]
            }
            None => message,
        };

        for split_message in split_message(message) {
            self.defer_ephemeral().await?;
            self.send(
                CreateReply::default()
                    .content(split_message)
                    .ephemeral(ephemeral),
            )
            .await?;
        }

        Ok(())
    }

    async fn say_ephemeral(&self, message: &str) -> Result<(), Error> {
        if message.len() == 0 {
            self.send(
                CreateReply::default()
                    .content(WARN_EMPTY_MESSAGE)
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }

        self.send(CreateReply::default().content(message).ephemeral(true))
            .await?;
        Ok(())
    }

    async fn say_long(&self, message: &str, ephemeral: bool) -> Result<(), Error> {
        if !ephemeral && message.len() > MESSAGE_BYTE_LIMIT {
            self.say_ephemeral(WARN_MESSAGE_SIZE_EXCEEDED).await?;
            return Ok(());
        } else if message.len() == 0 {
            self.say_ephemeral(WARN_EMPTY_MESSAGE).await?;
            return Ok(());
        }

        for m in split_long_string(&message) {
            self.send(CreateReply::default().content(m).ephemeral(ephemeral))
                .await?;
        }
        Ok(())
    }
}

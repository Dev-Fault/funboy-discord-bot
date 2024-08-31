use crate::{Context, Error};

use poise::CreateReply;

use super::discord_message_format::{format_output_vector, split_long_string, split_message};

pub trait ContextExtension {
    async fn say_vec(&self, message: Vec<String>, ephemeral: bool) -> Result<(), Error>;

    async fn say_ephemeral(&self, message: &str) -> Result<(), Error>;

    async fn multi_say(&self, message: &str, ephemeral: bool) -> Result<(), Error>;
}

impl<'a> ContextExtension for Context<'a> {
    async fn say_vec(&self, message: Vec<String>, ephemeral: bool) -> Result<(), Error> {
        for split_message in split_message(&format_output_vector(message)) {
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
        self.send(CreateReply::default().content(message).ephemeral(true))
            .await?;
        Ok(())
    }

    async fn multi_say(&self, message: &str, ephemeral: bool) -> Result<(), Error> {
        for m in split_long_string(&message) {
            self.send(CreateReply::default().content(m).ephemeral(ephemeral))
                .await?;
        }
        Ok(())
    }
}

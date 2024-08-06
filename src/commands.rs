use io_util::{format_output_vector, split_long_string, split_message};
use poise::{CreateReply, ReplyHandle};
use template_substitution_database::TemplateDatabase;
use tokio::sync::Mutex;

pub struct Data {
    pub t_db: Mutex<TemplateDatabase>,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

trait MessageHelper {
    async fn say_vec(&self, message: Vec<String>, ephemeral: bool) -> Result<(), Error>;

    async fn say_ephemeral(&self, message: &str) -> Result<(), Error>;

    async fn multi_say(&self, message: &str) -> Result<(), Error>;
}

impl<'a> MessageHelper for Context<'a> {
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

    async fn multi_say(&self, message: &str) -> Result<(), Error> {
        for m in split_long_string(&message) {
            self.send(CreateReply::default().content(m).ephemeral(false))
                .await?;
        }
        Ok(())
    }
}

mod io_util;
pub mod miscellaneous;
pub mod text_gen;

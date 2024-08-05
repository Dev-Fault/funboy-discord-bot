use template_substitution_database::TemplateDatabase;
use tokio::sync::Mutex;

pub struct Data {
    pub t_db: Mutex<TemplateDatabase>,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod io_util;
pub mod miscellaneous;
pub mod text_gen;

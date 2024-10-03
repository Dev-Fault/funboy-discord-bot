use std::str::FromStr;

use rand::distributions::uniform::SampleUniform;
use rand::Rng;

use crate::io_utils::discord_message_format::vectorize_input;
use crate::Context;
use crate::Error;

/// Generate a random number between a minimum and maximum value.
///
/// Example usage: **/random_number** min: **1** max: **6**
#[poise::command(slash_command, prefix_command)]
pub async fn random_number(ctx: Context<'_>, min: String, max: String) -> Result<(), Error> {
    if min.contains('.') || max.contains('.') {
        match get_random_result::<f64>(min, max) {
            Ok(result) => ctx.say(result).await?,

            Err(e) => ctx.say(format!("Error: {e}")).await?,
        };
    } else {
        match get_random_result::<i64>(min, max) {
            Ok(result) => ctx.say(result).await?,

            Err(e) => ctx.say(format!("Error: {e}")).await?,
        };
    }
    Ok(())
}

fn get_random_result<T: FromStr + PartialOrd + SampleUniform + ToString>(
    min: String,
    max: String,
) -> Result<String, &'static str> {
    match (min.parse(), max.parse()) {
        (Ok(min), Ok(max)) => Ok(get_random_inclusive::<T>(min, max).to_string()),
        _ => Err("Error: min and max values must be a number."),
    }
}

/// Select a random word from a list of words.
///
/// Use quotes for multi-word terms like: "apple tree"
///
/// Example usage: **/random_word** words: **"apple tree" pear "orange tree" apricot**
#[poise::command(slash_command, prefix_command)]
pub async fn random_word(ctx: Context<'_>, words: String) -> Result<(), Error> {
    let input = vectorize_input(&words);
    if input.len() < 2 {
        ctx.say(format!("Error: enter at least two entries."))
            .await?;
    } else {
        let output = input[get_random_inclusive(0, input.len() - 1)];
        ctx.say(output).await?;
    }
    Ok(())
}

pub fn get_random_inclusive<T: SampleUniform + PartialOrd>(min: T, max: T) -> T {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

pub fn get_random_exclusive<T: SampleUniform + PartialOrd>(min: T, max: T) -> T {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

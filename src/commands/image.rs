use std::borrow::Borrow;

use crate::{commands::random, io_utils::context_extension::ContextExtension, Context, Error};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ImgurResponse {
    data: Vec<ImgurImage>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ImgurImage {
    link: String,
    title: Option<String>,
}

/// Search for and display an image based on a query.
///
/// Example usage: **/search_image**  query: cat
#[poise::command(slash_command, prefix_command, category = "Image")]
pub async fn search_image(ctx: Context<'_>, query: String) -> Result<(), Error> {
    match ctx.data().imgur_client_id.borrow() {
        Some(id) => {
            let images = imgur_gallery_search(&query, id).await?.data;

            if images.len() == 0 {
                ctx.say_ephemeral(&format!("No images for query {} were found.", query))
                    .await?;
            } else {
                let i = random::get_random_exclusive(0, images.len());
                ctx.say(&images[i].link).await?;
            }
        }
        None => {
            ctx.say_ephemeral("Error: Couldn't load the imgur API.")
                .await?;
        }
    }
    Ok(())
}

async fn imgur_gallery_search(query: &str, client_id: &str) -> Result<ImgurResponse, Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Client-ID {}", client_id).parse()?);

    let url = format!("https://api.imgur.com/3/gallery/search?q={}", query);
    let mut response = client
        .get(&url)
        .headers(headers.clone())
        .send()
        .await?
        .json::<ImgurResponse>()
        .await?;

    if response.data.len() == 0 {
        let url = format!("https://api.imgur.com/3/gallery/search?q_any={}", query);
        response = client
            .get(&url)
            .headers(headers)
            .send()
            .await?
            .json::<ImgurResponse>()
            .await?;
    }

    Ok(response)
}

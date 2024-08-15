use std::sync::Arc;

use crate::{Context, Error};

use super::*;
use poise::CreateReply;
use serenity::http;
use tokio::task;

const MAX_MESSAGE_SIZE: usize = 2000;
const IMAGE_TYPES: [&str; 3] = [".png", ".gif", ".jpg"];

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

pub struct OutputLog {
    pub present: String,
    pub not_present: String,
}

impl OutputLog {
    pub fn from(user_input: Vec<&str>, changes: Vec<&str>) -> Self {
        OutputLog {
            present: Self::stringify(&user_input, |input| changes.contains(&input)),
            not_present: Self::stringify(&user_input, |input| !changes.contains(&input)),
        }
    }

    fn stringify(strings: &Vec<&str>, predicate: impl Fn(&str) -> bool) -> String {
        let mut output: String = strings
            .iter()
            .filter(|input| predicate(input))
            .map(|s| {
                if s.contains(' ') {
                    format!("{}{}{}", "**\"**", s, "**\"** ")
                } else {
                    s.to_string() + " "
                }
            })
            .collect();

        output.pop();
        output
    }
}

#[derive(Debug)]
pub struct QuoteFilter<'a> {
    pub quoted: Vec<&'a str>,
    pub unquoted: Vec<&'a str>,
}

impl<'a> QuoteFilter<'a> {
    pub fn from(input: &'a str) -> Self {
        const EMPTY: (&str, &str) = ("", "");

        let mut quoted: Vec<&str> = Vec::new();
        let mut unquoted: Vec<&str> = Vec::new();

        let mut first_split = input.split_once("\"");
        let mut second_split = first_split.unwrap_or(EMPTY).1.split_once("\"");
        let mut left_overs = "";

        if first_split == None {
            left_overs = input;
        }

        while first_split != None && second_split != None {
            Self::push_if_not_empty(&mut unquoted, first_split.unwrap_or(EMPTY).0.trim());
            Self::push_if_not_empty(&mut quoted, second_split.unwrap_or(EMPTY).0.trim());
            first_split = (second_split).unwrap_or(EMPTY).1.split_once("\"");
            left_overs = second_split.unwrap_or(EMPTY).1;
            second_split = (first_split).unwrap_or(EMPTY).1.split_once("\"");
        }

        Self::push_if_not_empty(&mut unquoted, left_overs.trim());

        QuoteFilter { quoted, unquoted }
    }

    fn push_if_not_empty<'b>(input: &mut Vec<&'b str>, value: &'b str) {
        if !value.is_empty() {
            input.push(value);
        }
    }
}

pub fn vectorize_input(input: &str) -> Vec<&str> {
    let quote_filter = &QuoteFilter::from(&input);

    let mut output: Vec<&str> = Vec::new();

    for quoted in &quote_filter.quoted {
        output.push(&quoted);
    }

    for unquoted in &quote_filter.unquoted {
        for word in unquoted.split_whitespace() {
            output.push(word);
        }
    }

    output
}

pub fn split_message(message: &Vec<String>) -> Vec<String> {
    let mut message_split: Vec<String> = Vec::new();

    let mut iter = message.iter();
    let mut message_part: String = String::default();

    while let Some(value) = iter.next() {
        if message_part.len() + value.len() <= MAX_MESSAGE_SIZE {
            message_part.push_str(value);
        } else {
            message_split.push(message_part);
            message_part = String::default();
            if value.len() <= MAX_MESSAGE_SIZE {
                message_part.push_str(value);
            } else {
                for sub_str in split_long_string(value) {
                    message_split.push(sub_str.to_string());
                }
            }
        }
    }

    if !message_part.is_empty() {
        message_split.push(message_part);
    }

    message_split
}

pub fn split_long_string<'a>(s: &'a str) -> Vec<&'a str> {
    let mut output = Vec::new();
    let blocks: usize = s.len() / MAX_MESSAGE_SIZE;

    for i in 0..blocks {
        output.push(&s[i * MAX_MESSAGE_SIZE..(i + 1) * MAX_MESSAGE_SIZE]);
    }

    if blocks * MAX_MESSAGE_SIZE < s.len() {
        output.push(&s[blocks * MAX_MESSAGE_SIZE..s.len()]);
    }

    output
}

pub fn format_output_vector(output: Vec<String>) -> Vec<String> {
    output
        .iter()
        .map(|s| {
            if s.contains(' ') {
                format!("{}{}{}", "**\"**", s, "**\"** ")
            } else {
                s.to_string() + " "
            }
        })
        .collect()
}

pub fn extract_image_urls<'a>(input: &'a str) -> Vec<&'a str> {
    let mut urls = Vec::new();
    for word in input.split_whitespace() {
        for image_type in IMAGE_TYPES {
            if word.contains("https://") && word.contains(image_type) {
                urls.push(word);
            }
        }
    }
    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_quote_input() {
        let input = String::from(
            "cat \"\" \"United States of America\" bear snake lion \"my mom\"  \"ten bulls\" dog goat",
        );

        dbg!(&vectorize_input(&input));

        assert_eq!(vectorize_input(&input).len(), 9);
    }

    #[test]
    fn no_quote_input() {
        let input = String::from("This is some input");

        assert_eq!(vectorize_input(&input).len(), 4);
    }

    #[test]
    fn split_a_long_string() {
        let mut long_string = String::with_capacity(23000);

        for _ in 0..23000 {
            long_string.push('0');
        }

        let split_string = split_long_string(&long_string);

        for s in split_string {
            assert!(s.len() <= super::MAX_MESSAGE_SIZE);
        }
    }

    #[test]
    fn split_a_long_message() {
        let mut message: Vec<String> = Vec::new();
        let mut long_string = String::with_capacity(23000);

        for _ in 0..23000 {
            long_string.push('0');
        }
        message.push(long_string);

        let mut regular_string = String::with_capacity(1000);
        let mut regular_string_2 = String::with_capacity(2000);
        let mut regular_string_3 = String::with_capacity(1999);
        let mut regular_string_4 = String::with_capacity(2001);

        for _ in 0..1000 {
            regular_string.push('1');
        }
        for _ in 0..2000 {
            regular_string_2.push('2');
        }
        for _ in 0..1999 {
            regular_string_3.push('3');
        }
        for _ in 0..2001 {
            regular_string_4.push('4');
        }

        message.push(regular_string);
        message.push(regular_string_2);
        message.push(regular_string_3);
        message.push(regular_string_4);

        for split in split_message(&message) {
            assert!(split.len() <= super::MAX_MESSAGE_SIZE);
        }
    }
}

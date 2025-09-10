use super::quote_filter::QuoteFilter;

pub const DISCORD_CHARACTER_LIMIT: usize = 2000;
const IMAGE_TYPES: [&str; 3] = [".png", ".gif", ".jpg"];

pub fn vectorize_input(input: &str) -> Vec<&str> {
    let quote_filter = &QuoteFilter::from(input);

    let mut output: Vec<&str> = Vec::new();

    for quoted in &quote_filter.quoted {
        output.push(quoted);
    }

    for unquoted in &quote_filter.unquoted {
        for word in unquoted.split_whitespace() {
            output.push(word);
        }
    }

    output
}

pub fn split_message(message: &[&str]) -> Vec<String> {
    let mut message_split: Vec<String> = Vec::new();

    let iter = message.iter();
    let mut message_part: String = String::default();

    for value in iter {
        if message_part.len() + value.len() <= DISCORD_CHARACTER_LIMIT {
            message_part.push_str(value);
        } else {
            message_split.push(message_part);
            message_part = String::default();
            if value.len() <= DISCORD_CHARACTER_LIMIT {
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

pub fn split_long_string(s: &str) -> Vec<&str> {
    let mut output = Vec::new();

    let mut output_length = 0;
    let mut message_length = 0;

    for word in s.split(' ').collect::<Vec<&str>>() {
        if message_length + word.len() + 1 > DISCORD_CHARACTER_LIMIT {
            output.push(
                s.get(output_length..output_length + message_length)
                    .unwrap_or_default(),
            );
            output_length += message_length;
            message_length = 0;
        }
        message_length += word.len() + 1;
    }

    output.push(
        s.get(output_length..output_length + message_length - 1)
            .unwrap_or_default(),
    );

    output
}

pub fn format_as_standard_list(output: &[&str]) -> Vec<String> {
    output
        .iter()
        .map(|s| {
            if s.len() > DISCORD_CHARACTER_LIMIT / 10 {
                "\n".to_string() + s + "\n"
            } else if s.contains(' ') {
                format!("{}{}{}", "[", s, "] ")
            } else {
                s.to_string() + " "
            }
        })
        .collect()
}

pub fn format_as_numeric_list(output: &[&str]) -> Vec<String> {
    let mut i = 0;
    output
        .iter()
        .map(|s| {
            let numbered = i.to_string() + ": " + s + "\n";
            i += 1;
            numbered
        })
        .collect()
}

pub fn extract_image_urls(input: &str) -> Vec<&str> {
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

        // dbg!(&vectorize_input(&input));

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
            assert!(s.len() <= super::DISCORD_CHARACTER_LIMIT);
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

        for split in split_message(&message.iter().map(|s| &s[..]).collect::<Vec<&str>>()[..]) {
            assert!(split.len() <= super::DISCORD_CHARACTER_LIMIT);
        }
    }
}

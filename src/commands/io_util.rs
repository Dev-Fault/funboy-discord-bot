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
                    format!("{}{}{}", "\"", s, "\" ")
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

#[cfg(test)]
mod tests {
    use crate::commands::vectorize_input;

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
}

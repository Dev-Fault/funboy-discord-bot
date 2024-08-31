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

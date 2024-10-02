use crate::text_interpolator::TemplateSplit;

pub const TEMPLATE_APOSTROPHE: char = '\'';
pub const TEMPLATE_CARROT: char = '^';
pub const TEMPLATE_BACK_TICK: char = '`';
pub const TEMPLATE_HEADERS: [char; 3] = [TEMPLATE_APOSTROPHE, TEMPLATE_CARROT, TEMPLATE_BACK_TICK];

/// Checks if a string is a template.
///
/// The default implementation considers a string starting with any character in the
/// TEMPLATE_HEADERS array to be a template.
///
/// Returns None if the string is empty or if the first character is not found in the
/// TEMPLATE_HEADERS array.
///
/// # Examples
///
/// ```
/// use text_interpolator::defaults::is_template;
///
/// let template = "'template";
/// let not_template = "not_template";
///
/// assert!(is_template(template));
/// assert!(!is_template(not_template));
/// ```
pub fn is_template(text: &str) -> bool {
    match text.chars().next() {
        Some(c) => TEMPLATE_HEADERS.contains(&c),
        None => false,
    }
}

/// Splits a templated string into it's prefix, template, and suffix components
///
/// When passing the function a templated string such as "^verb^ing" it will split anything prior
/// to the template character ^ into the prefix such that the prefix in the case will be "
/// Then it will split the text from the template character ^ up to any non alphanumeric character
/// such that the template portion in this case will be verb and the suffix portion will be ing"
///
/// # Examples
///
/// ```
/// use text_interpolator::TemplateSplit;
/// use text_interpolator::defaults::extract_template;
///
/// let template = "\"^verb^ing\"";
///
/// assert_eq!(TemplateSplit { prefix:"\"", template: "verb", suffix: "ing\"" }, extract_template(template));
/// ```
pub fn extract_template<'a>(embedded_template: &'a str) -> TemplateSplit<'a> {
    let prefix: &str;
    let template: &str;
    let suffix: &str;

    let mut split_char = ' ';

    match embedded_template.split_once(|c| TEMPLATE_HEADERS.contains(&c)) {
        Some(split) => match split.1.split_once(|c: char| {
            split_char = c;
            !c.is_alphanumeric()
        }) {
            Some(inner_split) => {
                prefix = split.0;
                template = inner_split.0;

                if TEMPLATE_HEADERS.contains(&split_char) {
                    suffix = inner_split.1;
                } else {
                    suffix = &split.1[template.len()..];
                }
            }
            None => {
                prefix = split.0;
                template = split.1;
                suffix = "";
            }
        },
        None => {
            prefix = "";
            template = "";
            suffix = "";
        }
    };

    TemplateSplit {
        prefix,
        template,
        suffix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_extration_with_prefix_and_suffix() {
        let extrated_template = extract_template("['adj.'..'.]");
        dbg!(&extrated_template);
        assert_eq!("[", extrated_template.prefix);
        assert_eq!(".'..'.]", extrated_template.suffix);
        assert_eq!("adj", extrated_template.template);
    }

    #[test]
    fn template_extration_with_punctuation_and_suffix() {
        let extrated_template = extract_template("'noun's");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("s", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_no_template() {
        let extrated_template = extract_template("noun");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("", extrated_template.suffix);
        assert_eq!("", extrated_template.template);
    }

    #[test]
    fn template_extration_with_ending_punctuation() {
        let extrated_template = extract_template("'noun.");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!(".", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_ending_punctuation_2() {
        let extrated_template = extract_template("'noun!");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("!", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_no_suffix_or_prefix() {
        let extrated_template = extract_template("'noun");
        dbg!(&extrated_template);

        assert_eq!("", extrated_template.prefix);
        assert_eq!("", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }

    #[test]
    fn template_extration_with_nested_template() {
        let extrated_template = extract_template("'noun'noun");
        dbg!(&extrated_template);
        assert_eq!("", extrated_template.prefix);
        assert_eq!("noun", extrated_template.suffix);
        assert_eq!("noun", extrated_template.template);
    }
}

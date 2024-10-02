//! # Text Interpolator
//!
//! `text_interpolator` is an object that takes input text that possibly contains templates (user
//! configurable) and maps them to substitutions.
//! To do so it uses modular functions to check if a word in the text is a
//! template, extract it, and then map it to it's substitute.
//!
//! It also supports nested templates requiring recursion to reach a valid substitute.

pub mod defaults;

use core::fmt;
use std::collections::HashSet;

use defaults::TEMPLATE_HEADERS;

#[derive(Debug, Clone)]
pub struct NestedTemplateLoopError;

impl fmt::Display for NestedTemplateLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "detected infinitely looping nested templates")
    }
}

#[derive(Debug, PartialEq)]
pub struct TemplateSplit<'a> {
    pub prefix: &'a str,
    pub template: &'a str,
    pub suffix: &'a str,
}

/// A function that determines if a string is or is not a template.
pub type IsTemplateFn = fn(&str) -> bool;
/// A function that splits a templated string into its suffix, prefix, and template components.
pub type ExtractTemplateFn = fn(&str) -> TemplateSplit;

#[derive(Debug)]
pub struct TextInterpolator {
    pub is_template: IsTemplateFn,
    pub extract_template: ExtractTemplateFn,
    template_set: HashSet<String>,
}

impl Default for TextInterpolator {
    /// Creates a TextInterpolator with default implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use text_interpolator::TextInterpolator;
    ///
    /// let text_interpolator = TextInterpolator::default();
    /// ```
    fn default() -> Self {
        TextInterpolator {
            is_template: defaults::is_template,
            extract_template: defaults::extract_template,
            template_set: HashSet::new(),
        }
    }
}

impl TextInterpolator {
    /// Creates a new TextInterpolator
    ///
    /// # Examples
    ///
    /// ```
    /// use text_interpolator::TextInterpolator;
    /// use text_interpolator::TemplateSplit;
    ///
    /// let text_interpolator = TextInterpolator::new(|_| true, |_|
    ///     TemplateSplit {
    ///         prefix: "",
    ///         template: "example",
    ///         suffix: "",
    ///     }
    /// );
    /// ```
    pub fn new(is_template: IsTemplateFn, extract_template: ExtractTemplateFn) -> Self {
        TextInterpolator {
            is_template,
            extract_template,
            template_set: HashSet::new(),
        }
    }

    /// Reconstructs a string by mapping any templated inner strings to a substitute.
    ///
    /// Will return an error if self referential templates are found to prevent infinite recursion.
    ///
    /// # Examples
    ///
    /// ```
    /// use text_interpolator::TextInterpolator;
    ///
    /// let mut text_interpolator = TextInterpolator::default();
    ///
    /// let text = text_interpolator.interp(
    ///     "The word 'template will be replaced with substitute",
    ///     &|s| {
    ///         if s == "template" {
    ///             Some("substitute".to_string())
    ///         } else {
    ///             None
    ///         }
    ///     },
    /// );
    ///
    /// assert_eq!(text.unwrap(), "The word substitute will be replaced with substitute");
    /// ```
    pub fn interp(
        &mut self,
        text: &str,
        map: &impl Fn(&str) -> Option<String>,
    ) -> Result<String, NestedTemplateLoopError> {
        // String will likely be at least as long as input
        let mut output = String::with_capacity(text.len());

        for item in text.split_whitespace() {
            let template_split = (self.extract_template)(item);

            match map(template_split.template) {
                Some(substitute) => {
                    if !self
                        .template_set
                        .insert(template_split.template.to_string())
                    {
                        self.template_set.clear();
                        return Err(NestedTemplateLoopError);
                    }

                    let mut substitution = substitute;

                    if self.contains_template(&substitution) {
                        substitution = self.interp(&substitution, map)?;
                    }

                    self.template_set.remove(template_split.template);

                    output.push_str(template_split.prefix);
                    output.push_str(&substitution);
                    output.push_str(template_split.suffix);
                    output.push(' ');
                }
                None => {
                    output.push_str(item);
                    output.push(' ');
                }
            }
        }

        // Remove trailing space
        output.pop();

        Ok(output)
    }

    /// Returns true if the string contains a template
    ///
    /// Returns false if it does not
    ///
    /// # Examples
    ///
    /// ```
    /// use text_interpolator::TextInterpolator;
    ///
    /// let text_interpolator = TextInterpolator::default();
    ///
    /// assert!(text_interpolator.contains_template("'template"));
    /// ```
    pub fn contains_template(&self, text: &str) -> bool {
        text.contains(TEMPLATE_HEADERS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_template(template: &str) -> Option<String> {
        match template {
            "verb" => Some(["run", "fall", "fly", "swim"][0].to_string()),
            "noun" => Some(["person", "place", "thing"][1].to_string()),
            "adj" => Some(["funny", "interesting", "aggrivating"][2].to_string()),
            "sentence" => Some(
                [
                    "A ^adj ^noun should never ^verb..",
                    "I have never seen someone ^verb with a ^noun before.",
                    "You are too ^adj to be ^adj...",
                ][1]
                .to_string(),
            ),
            "paragraph" => Some(["'sentence 'sentence 'sentence"][0].to_string()),
            "infinite" => Some("'infinite".to_string()),
            "nonexistantnest" => Some("'nothing".to_string()),
            _ => None,
        }
    }

    #[test]
    fn interpolate_non_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from(
            "This is an example of a basic input with no templates to be substituted.",
        );
        let interpolated_text = interpolator.interp(&text, &map_template).unwrap();

        dbg!(&interpolated_text);

        assert_eq!(&text, &interpolated_text);
    }

    #[test]
    fn interpolate_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("A 'adj 'noun will always 'verb in the morning.");
        let interpolated_text = interpolator.interp(&text, &map_template).unwrap();

        dbg!(&interpolated_text);

        assert!(!interpolated_text.contains("'adj"));
        assert!(!interpolated_text.contains("'noun"));
        assert!(!interpolated_text.contains("'verb"));
    }

    #[test]
    fn interpolate_templated_text_2() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("I'm 'verb'ing some 'adj 'noun's right now.");
        let interpolated_text = interpolator.interp(&text, &map_template).unwrap();

        dbg!(&interpolated_text);

        assert!(!interpolated_text.contains("'verb"));
        assert!(!interpolated_text.contains("'adj"));
        assert!(!interpolated_text.contains("'noun"));
    }

    #[test]
    fn interpolated_nested_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'sentence");
        let interpolated_text = interpolator.interp(&text, &map_template);
        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_double_nested_templated_text() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'paragraph");
        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn interpolated_double_nested_templated_text_with_prefix_and_suffix() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("My Story:'paragraph...");

        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert!(!interpolator.contains_template(&interpolated_text.unwrap()));
    }

    #[test]
    fn missing_template() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'klsfjkaejfaeskfjl");

        let interpolated_text = interpolator.interp(&text, &map_template);

        dbg!(&interpolated_text);

        assert_eq!("'klsfjkaejfaeskfjl", &interpolated_text.unwrap());
    }

    #[test]
    fn missing_nested_template() {
        let mut interpolator = TextInterpolator::default();
        let interp_text = interpolator.interp("'nonexistantnest", &map_template);
        dbg!(interp_text.unwrap());
    }

    #[test]
    fn infinite_self_recursion() {
        let mut interpolator = TextInterpolator::default();

        let text: String = String::from("'infinite");

        let interpolated_text = interpolator.interp(&text, &map_template);

        assert!(&interpolated_text.is_err());
    }
}

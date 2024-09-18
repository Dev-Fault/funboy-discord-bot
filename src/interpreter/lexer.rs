use core::panic;

pub const COMMA: &str = ",";
pub const OPENING_PARENTHESIS: &str = "(";
pub const CLOSING_PARENTHESIS: &str = ")";
pub const QUOTE: &str = "\"";
pub const ESCAPE: &str = "\\";
pub const ESCAPED_QUOTE: &str = "\\\"";

pub const SYMBOLS: [&str; 4] = [",", "(", ")", "\""];
pub const QUOTE_SYMBOLS: [&str; 2] = [ESCAPED_QUOTE, QUOTE];

#[derive(Debug, PartialEq)]
pub enum TokenType {
    OpeningQuote,
    ClosingQuote,
    OpeningParenthesis,
    ClosingParenthesis,
    Comma,
    Command,
    Text,
    Number,
    Identifier,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Option<String>,
}

pub fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();
    let mut inside_quote = false;
    let mut incomplete_string: Option<String> = None;

    for c in code.chars() {
        buffer.push(c);

        let symbols: &[&str];

        if inside_quote {
            symbols = &QUOTE_SYMBOLS;
        } else {
            symbols = &SYMBOLS;
        }

        if let Some(symbol) = get_symbol(&buffer, &symbols) {
            let (left, _right) = buffer.split_once(symbol).unwrap();

            match symbol {
                ESCAPED_QUOTE if inside_quote => {
                    let split = buffer.split_once(ESCAPE).unwrap();
                    incomplete_string =
                        Some(format!("{}{}", split.0.to_string(), split.1.to_string()));
                }
                QUOTE => {
                    if inside_quote {
                        let mut text = String::new();
                        inside_quote = false;

                        if let Some(ref s) = incomplete_string {
                            text.push_str(&s);
                            incomplete_string = None;
                        }

                        text.push_str(left);

                        tokens.push(Token {
                            token_type: TokenType::Text,
                            value: Some(text),
                        });
                        tokens.push(Token {
                            token_type: TokenType::ClosingQuote,
                            value: None,
                        });
                    } else {
                        inside_quote = true;

                        tokens.push(Token {
                            token_type: TokenType::OpeningQuote,
                            value: None,
                        });
                    }
                }
                OPENING_PARENTHESIS if !inside_quote => {
                    tokens.push(Token {
                        token_type: TokenType::Command,
                        value: Some(left.trim().to_string()),
                    });

                    tokens.push(Token {
                        token_type: TokenType::OpeningParenthesis,
                        value: None,
                    });
                }
                CLOSING_PARENTHESIS if !inside_quote => {
                    if buffer != CLOSING_PARENTHESIS {
                        let token_type;

                        if left.trim().parse::<f64>().is_ok() {
                            token_type = TokenType::Number
                        } else {
                            token_type = TokenType::Identifier
                        }

                        tokens.push(Token {
                            token_type,
                            value: Some(left.trim().to_string()),
                        });
                    }

                    tokens.push(Token {
                        token_type: TokenType::ClosingParenthesis,
                        value: None,
                    });
                }
                COMMA if !inside_quote => {
                    if buffer != COMMA {
                        let token_type;

                        if left.trim().parse::<f64>().is_ok() {
                            token_type = TokenType::Number
                        } else {
                            token_type = TokenType::Identifier
                        }

                        tokens.push(Token {
                            token_type,
                            value: Some(left.trim().to_string()),
                        });
                    }

                    tokens.push(Token {
                        token_type: TokenType::Comma,
                        value: None,
                    });
                }
                _ => {
                    if !inside_quote {
                        panic!("Should never contain an invalid symbol.")
                    } else {
                        continue;
                    }
                }
            }
            buffer.clear();
        }
    }

    tokens
}

fn get_symbol<'a>(code: &str, symbols: &[&'a str]) -> Option<&'a str> {
    for symbol in symbols {
        if code.contains(symbol) {
            return Some(symbol);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::interpreter::lexer::{tokenize, TokenType};

    #[test]
    fn standard_tokens() {
        let code = "add(\"'noun\", \"'puncuation\") multiply(2, identifier)";

        let tokens = tokenize(code);

        assert_eq!(tokens[0].token_type, TokenType::Command);
        assert_eq!(tokens[0].value, Some("add".to_string()));

        assert_eq!(tokens[1].token_type, TokenType::OpeningParenthesis);
        assert_eq!(tokens[1].value, None);

        assert_eq!(tokens[2].token_type, TokenType::OpeningQuote);
        assert_eq!(tokens[2].value, None);

        assert_eq!(tokens[3].token_type, TokenType::Text);
        assert_eq!(tokens[3].value, Some("'noun".to_string()));

        assert_eq!(tokens[4].token_type, TokenType::ClosingQuote);
        assert_eq!(tokens[4].value, None);

        assert_eq!(tokens[5].token_type, TokenType::Comma);
        assert_eq!(tokens[5].value, None);

        assert_eq!(tokens[6].token_type, TokenType::OpeningQuote);
        assert_eq!(tokens[6].value, None);

        assert_eq!(tokens[7].token_type, TokenType::Text);
        assert_eq!(tokens[7].value, Some("'puncuation".to_string()));

        assert_eq!(tokens[8].token_type, TokenType::ClosingQuote);
        assert_eq!(tokens[8].value, None);

        assert_eq!(tokens[9].token_type, TokenType::ClosingParenthesis);
        assert_eq!(tokens[9].value, None);

        assert_eq!(tokens[10].token_type, TokenType::Command);
        assert_eq!(tokens[10].value, Some("multiply".to_string()));

        assert_eq!(tokens[11].token_type, TokenType::OpeningParenthesis);
        assert_eq!(tokens[11].value, None);

        assert_eq!(tokens[12].token_type, TokenType::Number);
        assert_eq!(tokens[12].value, Some("2".to_string()));

        assert_eq!(tokens[13].token_type, TokenType::Comma);
        assert_eq!(tokens[13].value, None);

        assert_eq!(tokens[14].token_type, TokenType::Identifier);
        assert_eq!(tokens[14].value, Some("identifier".to_string()));

        assert_eq!(tokens[15].token_type, TokenType::ClosingParenthesis);
        assert_eq!(tokens[15].value, None);

        // dbg!(tokens);
    }

    #[test]
    fn symbols_inside_quotes() {
        let code = "print(\" example of, some, (), symbols \\\" inside, of quotes \")";

        let tokens = tokenize(code);

        assert_eq!(tokens[0].token_type, TokenType::Command);
        assert_eq!(tokens[0].value, Some("print".to_string()));

        assert_eq!(tokens[1].token_type, TokenType::OpeningParenthesis);
        assert_eq!(tokens[1].value, None);

        assert_eq!(tokens[2].token_type, TokenType::OpeningQuote);
        assert_eq!(tokens[2].value, None);

        assert_eq!(tokens[3].token_type, TokenType::Text);
        assert_eq!(
            tokens[3].value,
            Some(" example of, some, (), symbols \" inside, of quotes ".to_string())
        );

        assert_eq!(tokens[4].token_type, TokenType::ClosingQuote);
        assert_eq!(tokens[4].value, None);

        assert_eq!(tokens[5].token_type, TokenType::ClosingParenthesis);
        assert_eq!(tokens[5].value, None);

        // dbg!(tokens);
    }
}
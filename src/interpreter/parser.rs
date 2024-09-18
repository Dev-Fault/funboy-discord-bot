// Ideas:
// boolean types
// conditionals: if_then(true, do())
// text functions: if_then(starts_with(paste(noun), "a"), print("an"))

use std::str::FromStr;

use super::lexer::Token;
use super::lexer::TokenType;

pub const COMMAND_STACK_EXPECT: &str = "Command stack should have at least one command";
pub const TOKEN_VALUE_EXCEPT: &str = "Token must have value";

pub const ADD: &str = "add";
pub const SUBTRACT: &str = "sub";
pub const MULTIPLY: &str = "mul";
pub const DIVIDE: &str = "div";
pub const SELECT_RANDOM: &str = "select_random";
pub const RANDOM_RANGE: &str = "random_range";
pub const CAPITALIZE: &str = "capitalize";
pub const UPPER: &str = "upper";
pub const LOWER: &str = "lower";
pub const REMOVE_WHITESPACE: &str = "remove_whitespace";
pub const REPEAT: &str = "repeat";
pub const COPY: &str = "copy";
pub const PASTE: &str = "paste";
pub const PRINT: &str = "print";
pub const CONCATENATE: &str = "concat";

const COMMAND_COUNT: usize = 14;
const COMMANDS: [&str; COMMAND_COUNT] = [
    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
    SELECT_RANDOM,
    RANDOM_RANGE,
    CAPITALIZE,
    UPPER,
    LOWER,
    REPEAT,
    COPY,
    PASTE,
    PRINT,
    CONCATENATE,
];

#[derive(Debug, PartialEq, Clone)]
pub enum CommandType {
    Add,
    Subtract,
    Multiply,
    Divide,
    SelectRandom,
    RandomRange,
    Capitalize,
    Upper,
    Lower,
    RemoveWhitespace,
    Repeat,
    Copy,
    Paste,
    Print,
    Concatenate,
}

impl FromStr for CommandType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ADD => Ok(CommandType::Add),
            SUBTRACT => Ok(CommandType::Subtract),
            MULTIPLY => Ok(CommandType::Multiply),
            DIVIDE => Ok(CommandType::Divide),
            SELECT_RANDOM => Ok(CommandType::SelectRandom),
            RANDOM_RANGE => Ok(CommandType::RandomRange),
            CAPITALIZE => Ok(CommandType::Capitalize),
            UPPER => Ok(CommandType::Upper),
            LOWER => Ok(CommandType::Lower),
            REPEAT => Ok(CommandType::Repeat),
            COPY => Ok(CommandType::Copy),
            PASTE => Ok(CommandType::Paste),
            PRINT => Ok(CommandType::Print),
            REMOVE_WHITESPACE => Ok(CommandType::RemoveWhitespace),
            CONCATENATE => Ok(CommandType::Concatenate),
            _ => Err(format!("Invalid command {}", s)),
        }
    }
}

impl ToString for CommandType {
    fn to_string(&self) -> String {
        match self {
            CommandType::Add => ADD,
            CommandType::Subtract => SUBTRACT,
            CommandType::Multiply => MULTIPLY,
            CommandType::Divide => DIVIDE,
            CommandType::SelectRandom => SELECT_RANDOM,
            CommandType::RandomRange => RANDOM_RANGE,
            CommandType::Capitalize => CAPITALIZE,
            CommandType::Upper => UPPER,
            CommandType::Lower => LOWER,
            CommandType::Repeat => REPEAT,
            CommandType::Copy => COPY,
            CommandType::Paste => PASTE,
            CommandType::Print => PRINT,
            CommandType::RemoveWhitespace => REMOVE_WHITESPACE,
            CommandType::Concatenate => CONCATENATE,
        }
        .to_string()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    Text(String),
    Int(i64),
    Float(f64),
    Identifier(String),
    Command(Command),
    None,
}

impl ValueType {
    pub fn extract_float(&self) -> Option<f64> {
        match self {
            ValueType::Int(value) => Some(*value as f64),
            ValueType::Float(value) => Some(*value),
            _ => None,
        }
    }
    pub fn extract_int(&self) -> Option<i64> {
        match self {
            ValueType::Int(value) => Some(*value),
            _ => None,
        }
    }
}

impl ToString for ValueType {
    fn to_string(&self) -> String {
        match self {
            ValueType::Text(value) => value.to_string(),
            ValueType::Int(value) => value.to_string(),
            ValueType::Float(value) => value.to_string(),
            ValueType::Identifier(value) => value.to_string(),
            ValueType::Command(value) => value.command_type.to_string(),
            ValueType::None => "".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Command {
    pub command_type: CommandType,
    pub args: Vec<ValueType>,
}

impl Command {
    fn from(command_name: &str) -> Result<Command, String> {
        let command_type = CommandType::from_str(command_name)?;

        Ok(Command {
            command_type,
            args: Vec::new(),
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Command>, String> {
    let mut commands: Vec<Command> = Vec::new();

    let mut command_stack: Vec<Command> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        match token.token_type {
            TokenType::OpeningQuote => {
                const VALID_PRECEDENTS: [TokenType; 2] =
                    [TokenType::Comma, TokenType::OpeningParenthesis];

                if tokens.len() < 1 || !VALID_PRECEDENTS.contains(&tokens[i - 1].token_type) {
                    return Err(
                        "Opening quote must come after a comma or opening parenthesis".to_string(),
                    );
                } else if command_stack.len() == 0 {
                    return Err("Opening quote must be inside of a command".to_string());
                }
            }
            TokenType::ClosingQuote => {
                const VALID_PROCEDENTS: [TokenType; 2] =
                    [TokenType::Comma, TokenType::ClosingParenthesis];

                if tokens.len() <= (i + 1) || !VALID_PROCEDENTS.contains(&tokens[i + 1].token_type)
                {
                    return Err(
                        "Closing quote must come before a comma or closing parenthesis".to_string(),
                    );
                } else if tokens.len() < 1 || tokens[i - 1].token_type != TokenType::Text {
                    return Err("Closing quote must come after text".to_string());
                } else if command_stack.len() == 0 {
                    return Err("Closing quote must be inside of a command".to_string());
                }
            }
            TokenType::OpeningParenthesis => {
                if tokens.len() < 1 || tokens[i - 1].token_type != TokenType::Command {
                    return Err("Parenthesis must be preceeded by a command".to_string());
                }
            }
            TokenType::ClosingParenthesis => match command_stack.pop() {
                Some(command) => {
                    if command_stack.len() == 0 {
                        commands.push(command)
                    } else {
                        command_stack
                            .last_mut()
                            .expect(COMMAND_STACK_EXPECT)
                            .args
                            .push(ValueType::Command(command));
                    }
                }
                None => return Err("Unmatched parenthesis".to_string()),
            },
            TokenType::Comma => {
                if tokens.len() < 1 || command_stack.len() == 0 {
                    return Err("Comma must be inside a command".to_string());
                } else if tokens[i - 1].token_type == TokenType::OpeningParenthesis {
                    return Err("Comma must come after a command parameter".to_string());
                }
            }
            TokenType::Command => {
                let value = token.value.as_ref().expect(TOKEN_VALUE_EXCEPT).clone();

                if tokens.len() <= (i + 1)
                    || tokens[i + 1].token_type != TokenType::OpeningParenthesis
                {
                    return Err(format!(
                        "Command [{}] must come before an opening parenthesis",
                        value
                    ));
                } else {
                    command_stack.push(Command::from(&value)?);
                }
            }
            TokenType::Text => {
                let value = token.value.as_ref().expect(TOKEN_VALUE_EXCEPT).clone();

                if tokens.len() < 1 || tokens[i - 1].token_type != TokenType::OpeningQuote {
                    return Err(format!("Text [{}] must come after an opening quote", value));
                } else if tokens.len() == i || tokens[i + 1].token_type != TokenType::ClosingQuote {
                    return Err(format!("Text [{}] must come before a closing quote", value));
                } else if command_stack.len() == 0 {
                    return Err(format!("Text [{}] must be inside a command", value));
                } else {
                    command_stack
                        .last_mut()
                        .expect(COMMAND_STACK_EXPECT)
                        .args
                        .push(ValueType::Text(value));
                }
            }
            TokenType::Number => {
                const VALID_PRECEDENTS: [TokenType; 2] =
                    [TokenType::Comma, TokenType::OpeningParenthesis];

                let value = token.value.as_ref().expect(TOKEN_VALUE_EXCEPT).clone();

                if tokens.len() < 1 || !VALID_PRECEDENTS.contains(&tokens[i - 1].token_type) {
                    dbg!(&tokens[i - 1]);
                    return Err(format!(
                        "Number [{}] must come after a comma or opening parenthesis",
                        value
                    ));
                } else if command_stack.len() == 0 {
                    return Err(format!("Number [{}] must be inside of a command", value));
                } else {
                    if let Ok(value) = value.parse::<i64>() {
                        command_stack
                            .last_mut()
                            .expect(COMMAND_STACK_EXPECT)
                            .args
                            .push(ValueType::Int(value));
                    } else {
                        let value = value.parse::<f64>().expect("Number must be parsable");

                        command_stack
                            .last_mut()
                            .expect(COMMAND_STACK_EXPECT)
                            .args
                            .push(ValueType::Float(value));
                    }
                }
            }
            TokenType::Identifier => {
                const VALID_PRECEDENTS: [TokenType; 2] =
                    [TokenType::Comma, TokenType::OpeningParenthesis];

                let value = token.value.as_ref().expect(TOKEN_VALUE_EXCEPT).clone();

                if tokens.len() < 1 || !VALID_PRECEDENTS.contains(&tokens[i - 1].token_type) {
                    return Err(format!(
                        "Identifier [{}] must come after a comma or opening parenthesis",
                        value
                    ));
                } else if command_stack.len() == 0 {
                    return Err(format!(
                        "Identifier [{}] must be inside of a command",
                        value
                    ));
                } else {
                    command_stack
                        .last_mut()
                        .expect(COMMAND_STACK_EXPECT)
                        .args
                        .push(ValueType::Identifier(value));
                }
            }
        }
    }

    if command_stack.len() != 0 {
        return Err("Unmatched parenthesis".to_string());
    } else {
        Ok(commands)
    }
}

#[cfg(test)]
mod tests {

    use crate::interpreter::{
        lexer::tokenize,
        parser::{Command, CommandType, ValueType},
    };

    use super::parse;

    #[test]
    fn simple_parse() {
        let code = "add(5, 10)";

        let commands = parse(tokenize(code)).unwrap();

        assert_eq!(commands[0].command_type, CommandType::Add);
        assert_eq!(
            commands[0].args,
            vec![ValueType::Int(5), ValueType::Int(10)]
        );

        // dbg!(commands);
    }

    #[test]
    fn nested_parse() {
        let code = "add(add(identifier, \"text\"), add(add(1, 2), 2))";

        let commands = parse(tokenize(code)).unwrap();

        assert_eq!(commands[0].command_type, CommandType::Add);
        assert_eq!(
            commands[0].args,
            vec![
                ValueType::Command(Command {
                    command_type: CommandType::Add,
                    args: vec![
                        ValueType::Identifier("identifier".to_string()),
                        ValueType::Text("text".to_string())
                    ],
                }),
                ValueType::Command(Command {
                    command_type: CommandType::Add,
                    args: vec![
                        ValueType::Command(Command {
                            command_type: CommandType::Add,
                            args: vec![ValueType::Int(1), ValueType::Int(2)]
                        }),
                        ValueType::Int(2)
                    ]
                })
            ]
        );
        // dbg!(commands);
    }
}
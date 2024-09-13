use lexer::tokenize;
use parser::{
    parse, Command, CommandType, ValueType, ADD, CAPITALIZE, COPY, LOWER, MULTIPLY, PASTE,
    REMOVE_WHITESPACE, REPEAT, SELECT_RANDOM, SUBTRACT, UPPER,
};
use rand::{self, Rng};
use std::collections::HashMap;

#[allow(dead_code)]
mod lexer;
#[allow(dead_code)]
mod parser;

pub struct Interpreter {
    vars: HashMap<String, ValueType>,
    log: Vec<ValueType>,
    output: String,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            log: Vec::new(),
            output: String::new(),
        }
    }

    pub fn interpret(&mut self, code: &str) -> Result<(), String> {
        let commands = parse(tokenize(code))?;

        for command in commands {
            let value = self.eval_command(command)?;
            self.log.push(value);
        }

        Ok(())
    }

    fn eval_command(&mut self, command: Command) -> Result<ValueType, String> {
        let mut args: Vec<ValueType> = Vec::new();
        let mut args_contain_float = false;

        for arg in command.args {
            match arg {
                ValueType::Command(command) => {
                    args.push(self.eval_command(command)?);
                }
                ValueType::Text(_) => args.push(arg),
                ValueType::Int(_) => args.push(arg),
                ValueType::Float(_) => {
                    args_contain_float = true;
                    args.push(arg)
                }
                ValueType::Identifier(_) => args.push(arg),
                ValueType::None => args.push(arg),
            }
        }

        match command.command_type {
            CommandType::Add => {
                if args.len() < 2 {
                    return Err(format!("{} must have two or more arguments", ADD));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum += value,
                                None => return Err(format!("{} can only operate on numbers", ADD)),
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(format!("{} can only operate on numbers", ADD));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum += value,
                                None => return Err(format!("{} can only operate on numbers", ADD)),
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(format!("{} can only operate on numbers", ADD));
                    }
                }
            }
            CommandType::Subtract => {
                if args.len() < 2 {
                    return Err(format!("{} must have two or more arguments", SUBTRACT));
                } else if args_contain_float {
                    if let Some(mut diff) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => diff -= value,
                                None => {
                                    return Err(format!("{} can only operate on numbers", SUBTRACT))
                                }
                            }
                        }

                        Ok(ValueType::Float(diff))
                    } else {
                        return Err(format!("{} can only operate on numbers", SUBTRACT));
                    }
                } else {
                    if let Some(mut diff) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => diff -= value,
                                None => {
                                    return Err(format!("{} can only operate on numbers", SUBTRACT))
                                }
                            }
                        }

                        Ok(ValueType::Int(diff))
                    } else {
                        return Err(format!("{} can only operate on numbers", SUBTRACT));
                    }
                }
            }
            CommandType::Multiply => {
                if args.len() < 2 {
                    return Err(format!("{} must have two or more arguments", MULTIPLY));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum *= value,
                                None => {
                                    return Err(format!("{} can only operate on numbers", MULTIPLY))
                                }
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(format!("{} can only operate on numbers", MULTIPLY));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum *= value,
                                None => {
                                    return Err(format!("{} can only operate on numbers", MULTIPLY))
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(format!("{} can only operate on numbers", MULTIPLY));
                    }
                }
            }
            CommandType::Divide => {
                if args.len() < 2 {
                    return Err(format!("{} must have two or more arguments", MULTIPLY));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum /= value,
                                None => {
                                    return Err(format!("{} can only operate on numbers", MULTIPLY))
                                }
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(format!("{} can only operate on numbers", MULTIPLY));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum /= value,
                                None => {
                                    return Err(format!("{} can only operate on numbers", MULTIPLY))
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(format!("{} can only operate on numbers", MULTIPLY));
                    }
                }
            }
            CommandType::SelectRandom => {
                if args.len() < 2 {
                    Err(format!(
                        "{} must have at least two arguments",
                        SELECT_RANDOM
                    ))
                } else {
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0..args.len());
                    Ok(args[index].clone())
                }
            }
            CommandType::RandomRange => {
                if args.len() != 2 {
                    return Err(format!("{} must have exactly two arguments", UPPER));
                } else {
                    let mut rng = rand::thread_rng();
                    match &args[0] {
                        ValueType::Int(min) => match &args[1] {
                            ValueType::Int(max) => Ok(ValueType::Int(rng.gen_range(*min..=*max))),
                            ValueType::Float(max) => {
                                Ok(ValueType::Float(rng.gen_range((*min as f64)..=*max)))
                            }

                            _ => Err(format!("{} can only operate on numbers", UPPER)),
                        },
                        ValueType::Float(min) => match &args[1] {
                            ValueType::Int(max) => {
                                Ok(ValueType::Float(rng.gen_range(*min..=(*max as f64))))
                            }
                            ValueType::Float(max) => {
                                Ok(ValueType::Float(rng.gen_range(*min..=*max)))
                            }
                            _ => Err(format!("{} can only operate on numbers", UPPER)),
                        },
                        _ => Err(format!("{} can only operate on numbers", UPPER)),
                    }
                }
            }
            CommandType::Capitalize => {
                if args.len() != 1 {
                    return Err(format!("{} must have only one argument", CAPITALIZE));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => {
                            if text.len() > 0 {
                                return Ok(ValueType::Text(format!(
                                    "{}{}",
                                    text[0..1].to_uppercase(),
                                    text[1..].to_lowercase()
                                )));
                            } else {
                                return Ok(ValueType::Text("".to_string()));
                            }
                        }
                        _ => Err(format!("{} can only operate on text", CAPITALIZE)),
                    }
                }
            }
            CommandType::Upper => {
                if args.len() != 1 {
                    return Err(format!("{} must have only one argument", UPPER));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => Ok(ValueType::Text(text.to_uppercase())),
                        _ => Err(format!("{} can only operate on text", UPPER)),
                    }
                }
            }
            CommandType::Lower => {
                if args.len() != 1 {
                    return Err(format!("{} must have only one argument", LOWER));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => Ok(ValueType::Text(text.to_lowercase())),
                        _ => Err(format!("{} can only operate on text", LOWER)),
                    }
                }
            }
            CommandType::Repeat => Err(format!("{} command not implemented", REPEAT)),
            CommandType::Copy => {
                if args.len() != 2 {
                    return Err(format!("{} must have exactly two arguments", COPY));
                } else {
                    if let ValueType::Identifier(identifier) = &args[1] {
                        match &args[0] {
                            ValueType::Identifier(_) => Err(format!(
                                "The first argument of {} must not be an identifier",
                                COPY
                            )),
                            ValueType::None => Err(format!(
                                "The first argument of {} must not be of type None",
                                COPY
                            )),
                            _ => {
                                self.vars.insert(identifier.to_string(), args[0].clone());
                                return Ok(ValueType::None);
                            }
                        }
                    } else {
                        return Err(format!("Second argument of {} must be an identifier", COPY));
                    }
                }
            }
            CommandType::Paste => {
                if args.len() != 1 {
                    return Err(format!("{} must have only one argument", PASTE));
                } else {
                    match &args[0] {
                        ValueType::Identifier(identifier) => match self.vars.get(identifier) {
                            Some(value) => Ok(value.clone()),
                            None => Err(format!("No identifier exists named {}", identifier)),
                        },
                        _ => Err(format!("Argument to {} must be an identifier", PASTE)),
                    }
                }
            }
            CommandType::Print => {
                for arg in args {
                    self.output.push_str(&arg.to_string());
                }
                Ok(ValueType::None)
            }
            CommandType::RemoveWhitespace => {
                if args.len() != 1 {
                    return Err(format!("{} must only have one argument", REMOVE_WHITESPACE));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => {
                            Ok(ValueType::Text(text.split_whitespace().collect()))
                        }
                        _ => Err(format!("{} can only operate on text", REMOVE_WHITESPACE)),
                    }
                }
            }
            CommandType::Concatenate => {
                let mut output = String::new();

                for arg in args {
                    output.push_str(&arg.to_string());
                }

                Ok(ValueType::Text(output))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::interpreter::{parser::ValueType, Interpreter};

    #[test]
    fn interpret_add() {
        let code = "add(5, 5) add(5.1, 5.2) add(1.0, 1.0) add(1.0, 1)";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Int(10));
        assert_eq!(interpreter.log[1], ValueType::Float(10.3));
        assert_eq!(interpreter.log[2], ValueType::Float(2.0));
        assert_eq!(interpreter.log[3], ValueType::Float(2.0));
    }

    #[test]
    fn interpret_subtract() {
        let code = "sub(-5, 5, -2) sub(4.2, 1.2) sub(1.0, 1.0) sub(1.0, 1)";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Int(-8));
        let value = interpreter.log[1].extract_float().unwrap();
        assert!((value < 3.01) & (value > 2.99));
        assert_eq!(interpreter.log[2], ValueType::Float(0.0));
        assert_eq!(interpreter.log[3], ValueType::Float(0.0));
    }

    #[test]
    fn interpret_multiply() {
        let code = "mul(2, 5) mul(2.0, 5)";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Int(10));
        assert_eq!(interpreter.log[1], ValueType::Float(10.0));
    }

    #[test]
    fn interpret_divide() {
        let code = "div(2, 3) div(3, 2) div (5.0, 2.5)";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Int(0));
        assert_eq!(interpreter.log[1], ValueType::Int(1));
        let value = interpreter.log[2].extract_float().unwrap();
        assert!((value < 2.01) & (value > 1.99));
    }

    #[test]
    fn interpret_print() {
        let code = "print(5, \"text\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.output, "5text");
    }

    #[test]
    fn interpret_concatenate() {
        let code = "concat(5, \"text\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("5text".to_string()));
    }

    #[test]
    fn interpret_capitalize() {
        let code = "capitalize(\"text\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("Text".to_string()));

        let code = "capitalize(\"t\")";
        interpreter.interpret(code).unwrap();
        assert_eq!(interpreter.log[1], ValueType::Text("T".to_string()));

        let code = "capitalize(\"\")";
        interpreter.interpret(code).unwrap();
        assert_eq!(interpreter.log[2], ValueType::Text("".to_string()));
    }

    #[test]
    fn interpret_upper() {
        let code = "upper(\"text\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("TEXT".to_string()));
    }

    #[test]
    fn interpret_lower() {
        let code = "lower(\"TEXT\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("text".to_string()));
    }

    #[test]
    fn interpret_remove_whitespace() {
        let code = "remove_whitespace(\"text with whitespace removed\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(
            interpreter.log[0],
            ValueType::Text("textwithwhitespaceremoved".to_string())
        );
    }

    #[test]
    fn interpret_copy_paste() {
        let code = "copy(\"text\", text) print(paste(text))";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.output, "text");
    }

    #[test]
    fn interpret_random_range_int() {
        let code = "random_range(5, 10)";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        match interpreter.log[0] {
            ValueType::Int(value) => {
                dbg!(value);
            }
            _ => panic!("Value type should be int"),
        }
    }

    #[test]
    fn interpret_random_range_float() {
        let code = "random_range(5.0, 10)";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        match interpreter.log[0] {
            ValueType::Float(value) => {
                dbg!(value);
            }
            _ => panic!("Value type should be float"),
        }
    }

    #[test]
    fn interpret_select_random() {
        let code = "print(select_random(\"apple\", \"orange\", 3, 6.7))";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert!(interpreter.output.len() > 0);
        dbg!(interpreter.output);
    }
}

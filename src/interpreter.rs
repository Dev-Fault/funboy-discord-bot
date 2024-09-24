use lexer::tokenize;
use parser::{
    parse, Command, CommandType, ValueType, ADD, AND, CAPITALIZE, COPY, DIVIDE, ENDS_WITH, EQ, GT,
    IF_THEN, IF_THEN_ELSE, LOWER, LT, MULTIPLY, NOT, OR, PASTE, REMOVE_WHITESPACE, REPEAT,
    SELECT_RANDOM, STARTS_WITH, SUBTRACT, UPPER,
};
use rand::{self, Rng};
use std::collections::HashMap;

#[allow(dead_code)]
mod lexer;
#[allow(dead_code)]
mod parser;

#[derive(Debug)]
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

    pub fn interpret(&mut self, code: &str) -> Result<String, String> {
        let commands = parse(tokenize(code))?;

        for command in commands {
            let value = self.eval_command(command)?;
            self.log.push(value);
        }

        Ok(self.output.drain(..).collect())
    }

    fn eval_command(&mut self, command: Command) -> Result<ValueType, String> {
        let mut args: Vec<ValueType> = Vec::new();
        let mut args_contain_float = false;
        let mut i = 0;

        for arg in command.args {
            match arg {
                ValueType::Command(ref command) => {
                    match command.command_type {
                        CommandType::IfThen if i == 1 => args.push(arg),
                        CommandType::IfThenElse if i == 1 || i == 2 => args.push(arg),
                        CommandType::Repeat if i == 1 => args.push(arg),
                        _ => args.push(self.eval_command(command.clone())?),
                    };
                }
                ValueType::Text(_) => args.push(arg),
                ValueType::Int(_) => args.push(arg),
                ValueType::Float(_) => {
                    args_contain_float = true;
                    args.push(arg)
                }
                ValueType::Identifier(_) => args.push(arg),
                ValueType::None => args.push(arg),
                ValueType::Bool(_) => args.push(arg),
            }
            i += 1;
        }

        match command.command_type {
            CommandType::Add => {
                if args.len() < 2 {
                    return Err(format!("command {} must have two or more arguments", ADD));
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
                        return Err(format!("command {} can only operate on numbers", ADD));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum += value,
                                None => {
                                    return Err(format!(
                                        "command {} can only operate on numbers",
                                        ADD
                                    ))
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(format!("command {} can only operate on numbers", ADD));
                    }
                }
            }
            CommandType::Subtract => {
                if args.len() < 2 {
                    return Err(format!(
                        "command {} must have two or more arguments",
                        SUBTRACT
                    ));
                } else if args_contain_float {
                    if let Some(mut diff) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => diff -= value,
                                None => {
                                    return Err(format!(
                                        "command {} can only operate on numbers",
                                        SUBTRACT
                                    ))
                                }
                            }
                        }

                        Ok(ValueType::Float(diff))
                    } else {
                        return Err(format!("command {} can only operate on numbers", SUBTRACT));
                    }
                } else {
                    if let Some(mut diff) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => diff -= value,
                                None => {
                                    return Err(format!(
                                        "command {} can only operate on numbers",
                                        SUBTRACT
                                    ))
                                }
                            }
                        }

                        Ok(ValueType::Int(diff))
                    } else {
                        return Err(format!("command {} can only operate on numbers", SUBTRACT));
                    }
                }
            }
            CommandType::Multiply => {
                if args.len() < 2 {
                    return Err(format!(
                        "command {} must have two or more arguments",
                        MULTIPLY
                    ));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum *= value,
                                None => {
                                    return Err(format!(
                                        "command {} can only operate on numbers",
                                        MULTIPLY
                                    ))
                                }
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(format!("command {} can only operate on numbers", MULTIPLY));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum *= value,
                                None => {
                                    return Err(format!(
                                        "command {} can only operate on numbers",
                                        MULTIPLY
                                    ))
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(format!("command {} can only operate on numbers", MULTIPLY));
                    }
                }
            }
            CommandType::Divide => {
                if args.len() < 2 {
                    return Err(format!(
                        "command {} must have two or more arguments",
                        DIVIDE
                    ));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum /= value,
                                None => {
                                    return Err(format!(
                                        "command {} can only operate on numbers",
                                        DIVIDE
                                    ))
                                }
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(format!("command {} can only operate on numbers", DIVIDE));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum /= value,
                                None => {
                                    return Err(format!(
                                        "command {} can only operate on numbers",
                                        DIVIDE
                                    ))
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(format!("command {} can only operate on numbers", DIVIDE));
                    }
                }
            }
            CommandType::SelectRandom => {
                if args.len() < 2 {
                    Err(format!(
                        "command {} must have at least two arguments",
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
                    return Err(format!("command {} must have exactly two arguments", UPPER));
                } else {
                    let mut rng = rand::thread_rng();
                    match &args[0] {
                        ValueType::Int(min) => match &args[1] {
                            ValueType::Int(max) => Ok(ValueType::Int(rng.gen_range(*min..=*max))),
                            ValueType::Float(max) => {
                                Ok(ValueType::Float(rng.gen_range((*min as f64)..=*max)))
                            }

                            _ => Err(format!("command {} can only operate on numbers", UPPER)),
                        },
                        ValueType::Float(min) => match &args[1] {
                            ValueType::Int(max) => {
                                Ok(ValueType::Float(rng.gen_range(*min..=(*max as f64))))
                            }
                            ValueType::Float(max) => {
                                Ok(ValueType::Float(rng.gen_range(*min..=*max)))
                            }
                            _ => Err(format!("command {} can only operate on numbers", UPPER)),
                        },
                        _ => Err(format!("command {} can only operate on numbers", UPPER)),
                    }
                }
            }
            CommandType::Capitalize => {
                if args.len() != 1 {
                    return Err(format!(
                        "command {} must have only one argument",
                        CAPITALIZE
                    ));
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
                        _ => Err(format!("command {} can only operate on text", CAPITALIZE)),
                    }
                }
            }
            CommandType::Upper => {
                if args.len() != 1 {
                    return Err(format!("command {} must have only one argument", UPPER));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => Ok(ValueType::Text(text.to_uppercase())),
                        _ => Err(format!("command {} can only operate on text", UPPER)),
                    }
                }
            }
            CommandType::Lower => {
                if args.len() != 1 {
                    return Err(format!("command {} must have only one argument", LOWER));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => Ok(ValueType::Text(text.to_lowercase())),
                        _ => Err(format!("command {} can only operate on text", LOWER)),
                    }
                }
            }
            CommandType::Repeat => Err(format!("command {} command not implemented", REPEAT)),
            CommandType::Copy => {
                if args.len() != 2 {
                    return Err(format!("command {} must have exactly two arguments", COPY));
                } else {
                    if let ValueType::Identifier(identifier) = &args[1] {
                        match &args[0] {
                            ValueType::Identifier(_) => Err(format!(
                                "The first argument of command {} must not be an identifier",
                                COPY
                            )),
                            ValueType::None => Err(format!(
                                "The first argument of command {} must not be of type None",
                                COPY
                            )),
                            _ => {
                                self.vars.insert(identifier.to_string(), args[0].clone());
                                return Ok(ValueType::None);
                            }
                        }
                    } else {
                        return Err(format!(
                            "Second argument of command {} must be an identifier",
                            COPY
                        ));
                    }
                }
            }
            CommandType::Paste => {
                if args.len() != 1 {
                    return Err(format!("command {} must have only one argument", PASTE));
                } else {
                    match &args[0] {
                        ValueType::Identifier(identifier) => match self.vars.get(identifier) {
                            Some(value) => Ok(value.clone()),
                            None => Err(format!("No identifier exists named {}", identifier)),
                        },
                        _ => Err(format!(
                            "argument to command {} must be an identifier",
                            PASTE
                        )),
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
                    return Err(format!(
                        "command {} must only have one argument",
                        REMOVE_WHITESPACE
                    ));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => {
                            Ok(ValueType::Text(text.split_whitespace().collect()))
                        }
                        _ => Err(format!(
                            "command {} can only operate on text",
                            REMOVE_WHITESPACE
                        )),
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
            CommandType::IfThen => {
                if args.len() != 2 {
                    return Err(format!("command {} must have two arguments", IF_THEN));
                } else {
                    match &args[0] {
                        ValueType::Bool(bool) => {
                            if *bool {
                                match &args[1] {
                                    ValueType::Command(value) => self.eval_command(value.clone()),
                                    _ => Ok(args[1].clone()),
                                }
                            } else {
                                Ok(ValueType::None)
                            }
                        }
                        _ => Err(format!(
                            "first argument of command {} must evaluate to a boolean value",
                            IF_THEN
                        )),
                    }
                }
            }
            CommandType::IfThenElse => {
                if args.len() != 3 {
                    return Err(format!(
                        "command {} must have three arguments",
                        IF_THEN_ELSE
                    ));
                } else {
                    match &args[0] {
                        ValueType::Bool(bool) => {
                            if *bool {
                                match &args[1] {
                                    ValueType::Command(value) => self.eval_command(value.clone()),
                                    _ => Ok(args[1].clone()),
                                }
                            } else {
                                match &args[2] {
                                    ValueType::Command(value) => self.eval_command(value.clone()),
                                    _ => Ok(args[2].clone()),
                                }
                            }
                        }
                        _ => Err(format!(
                            "first argument of command {} must evaluate to a boolean value",
                            IF_THEN_ELSE
                        )),
                    }
                }
            }
            CommandType::Not => {
                if args.len() != 1 {
                    return Err(format!("command {} must only have one argument", NOT));
                } else {
                    match &args[0] {
                        ValueType::Bool(bool) => Ok(ValueType::Bool(!*bool)),
                        _ => Err(format!(
                            "first argument of command {} must evaluate to a boolean value",
                            NOT
                        )),
                    }
                }
            }
            CommandType::And => {
                if args.len() < 2 {
                    return Err(format!("command {} must have at least two arguments", AND));
                }
                let mut value: bool = true;
                for arg in args {
                    match arg {
                        ValueType::Bool(bool) => value = value & bool,
                        _ => return Err(format!("command {} only works on boolean values", AND)),
                    }
                }
                Ok(ValueType::Bool(value))
            }
            CommandType::Or => {
                if args.len() < 2 {
                    return Err(format!("command {} must have at least two arguments", OR));
                }
                let mut value: bool = false;
                for arg in args {
                    match arg {
                        ValueType::Bool(bool) => value = value | bool,
                        _ => return Err(format!("command {} only works on boolean values", OR)),
                    }
                }
                Ok(ValueType::Bool(value))
            }
            CommandType::Eq => {
                if args.len() != 2 {
                    return Err(format!("command {} must have two arguments", EQ));
                } else {
                    match (&args[0], &args[1]) {
                        (ValueType::Text(value_a), ValueType::Text(value_b)) => {
                            Ok(ValueType::Bool(&value_a[..] == &value_b[..]))
                        }
                        (ValueType::Int(value_a), ValueType::Int(value_b)) => {
                            Ok(ValueType::Bool(*value_a == *value_b))
                        }
                        (ValueType::Int(value_a), ValueType::Float(value_b)) => {
                            Ok(ValueType::Bool((*value_a as f64) == *value_b))
                        }
                        (ValueType::Float(value_a), ValueType::Int(value_b)) => {
                            Ok(ValueType::Bool(*value_a == (*value_b as f64)))
                        }
                        (ValueType::Float(value_a), ValueType::Float(value_b)) => {
                            Ok(ValueType::Bool(
                                *value_a < *value_b + 0.0001 && *value_a > *value_b - 0.0001,
                            ))
                        }
                        (ValueType::Bool(value_a), ValueType::Bool(value_b)) => {
                            Ok(ValueType::Bool(*value_a == *value_b))
                        }
                        _ => Err(format!(
                            "Cannot compare {} with {}",
                            args[0].to_string(),
                            args[1].to_string()
                        )),
                    }
                }
            }
            CommandType::Gt => {
                if args.len() != 2 {
                    return Err(format!("command {} must have two arguments", GT));
                } else {
                    match (&args[0], &args[1]) {
                        (ValueType::Int(value_a), ValueType::Int(value_b)) => {
                            Ok(ValueType::Bool(*value_a > *value_b))
                        }
                        (ValueType::Int(value_a), ValueType::Float(value_b)) => {
                            Ok(ValueType::Bool((*value_a as f64) > *value_b))
                        }
                        (ValueType::Float(value_a), ValueType::Int(value_b)) => {
                            Ok(ValueType::Bool(*value_a > (*value_b as f64)))
                        }
                        (ValueType::Float(value_a), ValueType::Float(value_b)) => {
                            Ok(ValueType::Bool(*value_a > *value_b))
                        }
                        _ => Err(format!("command {} only works on numbers", GT)),
                    }
                }
            }
            CommandType::Lt => {
                if args.len() != 2 {
                    return Err(format!("command {} must have two arguments", LT));
                } else {
                    match (&args[0], &args[1]) {
                        (ValueType::Int(value_a), ValueType::Int(value_b)) => {
                            Ok(ValueType::Bool(*value_a < *value_b))
                        }
                        (ValueType::Int(value_a), ValueType::Float(value_b)) => {
                            Ok(ValueType::Bool((*value_a as f64) < *value_b))
                        }
                        (ValueType::Float(value_a), ValueType::Int(value_b)) => {
                            Ok(ValueType::Bool(*value_a < (*value_b as f64)))
                        }
                        (ValueType::Float(value_a), ValueType::Float(value_b)) => {
                            Ok(ValueType::Bool(*value_a < *value_b))
                        }
                        _ => Err(format!("command {} only works on numbers", LT)),
                    }
                }
            }
            CommandType::StartsWith => {
                if args.len() != 2 {
                    return Err(format!("command {} must have two arguments", STARTS_WITH));
                } else {
                    match (&args[0], &args[1]) {
                        (ValueType::Text(value_a), ValueType::Text(value_b)) => {
                            Ok(ValueType::Bool(value_a.starts_with(value_b)))
                        }
                        _ => Err(format!("command {} only works on text", STARTS_WITH)),
                    }
                }
            }
            CommandType::EndsWith => {
                if args.len() != 2 {
                    return Err(format!("command {} must have two arguments", ENDS_WITH));
                } else {
                    match (&args[0], &args[1]) {
                        (ValueType::Text(value_a), ValueType::Text(value_b)) => {
                            Ok(ValueType::Bool(value_a.ends_with(value_b)))
                        }
                        _ => Err(format!("command {} only works on text", ENDS_WITH)),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::interpreter::{parser::ValueType, Interpreter};

    #[test]
    fn interpret_if_then() {
        let code = "if_then(true, \"true\") if_then(false, \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::None);
    }

    #[test]
    fn interpret_if_then_else() {
        let code = "if_then_else(false, \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
    }

    #[test]
    fn interpret_not() {
        let code = "if_then_else(not(false), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_and() {
        let code = "if_then_else(and(true, false), \"true\", \"false\") if_then_else(and(true, true), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_or() {
        let code = "if_then_else(or(true, false), \"true\", \"false\") if_then_else(or(true, true), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_eq() {
        let code = "if_then_else(eq(1, 1), \"true\", \"false\") if_then_else(eq(1, 2), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("false".to_string()));
    }

    #[test]
    fn interpret_gt() {
        let code = "if_then_else(gt(1, 1), \"true\", \"false\") if_then_else(gt(2, 1), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_lt() {
        let code = "if_then_else(lt(1, 1), \"true\", \"false\") if_then_else(lt(1, 2), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_starts_with() {
        let code = "if_then_else(starts_with(\"apple\", \"a\"), \"true\", \"false\") if_then_else(starts_with(\"apple\", \"b\"), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("false".to_string()));
    }

    #[test]
    fn interpret_ends_with() {
        let code = "if_then_else(ends_with(\"apple\", \"e\"), \"true\", \"false\") if_then_else(ends_with(\"apple\", \"b\"), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("false".to_string()));
    }

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
        let output = interpreter.interpret(code).unwrap();

        assert_eq!(output, "5text");
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
        let output = interpreter.interpret(code).unwrap();

        assert_eq!(output, "text");
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
        let output = interpreter.interpret(code).unwrap();

        assert!(output.len() > 0);
        dbg!(output);
    }
}

use crate::TemplateDatabase;
use lexer::tokenize;
use parser::{parse, Command, CommandType, ValueType};
use rand::{self, Rng};
use std::collections::HashMap;
use text_interpolator::{defaults::TEMPLATE_CARROT, TextInterpolator};

use crate::{io_utils::context_extension::MESSAGE_BYTE_LIMIT, FUNBOY_DB_PATH};

#[allow(dead_code)]
mod lexer;
#[allow(dead_code)]
mod parser;

const REPEAT_LIMIT: u16 = u16::MAX;
const VAR_MAP_BYTE_LIMIT: usize = 65535 * 100;
const OUTPUT_BYTE_LIMIT: usize = MESSAGE_BYTE_LIMIT;

const ERROR_NO_ARGS: &str = "takes no arguments";
const ERROR_TWO_OR_MORE_ARGS: &str = "must have two or more arguments";
const ERROR_EXACTLY_ONE_ARG: &str = "must have exactly one argument";
const ERROR_EXACTLY_TWO_ARGS: &str = "must have exactly two arguments";
const ERROR_EXACTLY_THREE_ARGS: &str = "must have exactly three arguments";
const ERROR_ARGS_MUST_BE_NUMBER: &str = "all arguments must be of type Number";
const ERROR_ARGS_MUST_BE_BOOL: &str = "all arguments must be of type Bool";
const ERROR_ARGS_MUST_BE_TEXT: &str = "all arguments must be of type Text";
const ERROR_ARG_MUST_BE_TEXT: &str = "argument must be of type Text";
const ERROR_ARG_MUST_BE_NUMBER: &str = "argument must be of type Number";
const ERROR_ARG_MUST_BE_IDENTIFIER: &str = "argument must be of type Identifier";
const ERROR_ARG_ONE_MUST_BE_WHOLE_NUMBER: &str = "first argument must be a whole number";
const ERROR_ARG_ONE_MUST_BE_BOOL: &str = "first argument must be of type Bool";
const ERROR_ARGS_AFTER_ARG_ONE_MUST_BE_COMMAND: &str =
    "arguments following first argument must be of type Command";
const ERROR_ARG_ONE_MUST_NOT_BE_IDENTIFIER: &str = "first argument must not be of type Identifier";
const ERROR_ARG_ONE_MUST_NOT_BE_NONE: &str = "first argument must not be of type None";
const ERROR_ARG_TWO_MUST_BE_IDENTIFIER: &str = "second argument must be of type Identifier";
const ERROR_UNKNOWN_IDENTIFIER: &str = "No identifier exists named";

#[derive(Debug)]
pub struct VarMap {
    data: HashMap<String, ValueType>,
    size: usize,
}

impl VarMap {
    pub fn new() -> Self {
        VarMap {
            data: HashMap::new(),
            size: 0,
        }
    }

    pub fn insert_var(&mut self, name: String, value: ValueType) -> Result<(), String> {
        if self.size.saturating_add(value.get_size()) <= VAR_MAP_BYTE_LIMIT {
            self.data.insert(name, value);
            Ok(())
        } else {
            Err(format!(
                "interpreter memory limit of {} bytes exceeded",
                VAR_MAP_BYTE_LIMIT
            ))
        }
    }

    pub fn get_var(&mut self, name: &String) -> Option<&mut ValueType> {
        self.data.get_mut(name)
    }
}

#[derive(Debug)]
pub struct Interpreter {
    vars: VarMap,
    output: String,
    log: Vec<ValueType>,
    db: TemplateDatabase,
    interpolator: TextInterpolator,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            vars: VarMap::new(),
            output: String::new(),
            log: Vec::new(),
            db: TemplateDatabase::from_path(FUNBOY_DB_PATH)
                .expect("Funboy database failed to load."),
            interpolator: TextInterpolator::default(),
        }
    }

    pub fn interpret(&mut self, code: &str) -> Result<String, String> {
        let commands = parse(tokenize(code))?;

        for command in commands {
            self.eval_command(command)?;
        }

        Ok(self.output.drain(..).collect())
    }

    pub fn interpret_and_log(&mut self, code: &str) -> Result<String, String> {
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
                ValueType::Command(ref sub_command) => {
                    match command.command_type {
                        CommandType::IfThen if i == 1 => args.push(arg),
                        CommandType::IfThenElse if i == 1 || i == 2 => args.push(arg),
                        CommandType::Repeat if i != 0 => args.push(arg),
                        _ => args.push(self.eval_command(sub_command.clone())?),
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

        let command_type = command.command_type;

        match command_type {
            CommandType::Add => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum += value,
                                None => return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER)),
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum += value,
                                None => {
                                    return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                }
            }
            CommandType::Subtract => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                } else if args_contain_float {
                    if let Some(mut diff) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => diff -= value,
                                None => {
                                    return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                                }
                            }
                        }

                        Ok(ValueType::Float(diff))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                } else {
                    if let Some(mut diff) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => diff -= value,
                                None => {
                                    return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                                }
                            }
                        }

                        Ok(ValueType::Int(diff))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                }
            }
            CommandType::Multiply => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum *= value,
                                None => {
                                    return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                                }
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum *= value,
                                None => {
                                    return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                }
            }
            CommandType::Divide => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                } else if args_contain_float {
                    if let Some(mut sum) = args[0].extract_float() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_float() {
                                Some(value) => sum /= value,
                                None => {
                                    return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                                }
                            }
                        }

                        Ok(ValueType::Float(sum))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                } else {
                    if let Some(mut sum) = args[0].extract_int() {
                        for arg in &args[1..args.len()] {
                            match arg.extract_int() {
                                Some(value) => sum /= value,
                                None => {
                                    return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                                }
                            }
                        }

                        Ok(ValueType::Int(sum))
                    } else {
                        return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER));
                    }
                }
            }
            CommandType::SelectRandom => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                } else {
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0..args.len());
                    Ok(args[index].clone())
                }
            }
            CommandType::RandomRange => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
                } else {
                    let mut rng = rand::thread_rng();
                    match &args[0] {
                        ValueType::Int(min) => match &args[1] {
                            ValueType::Int(max) => Ok(ValueType::Int(rng.gen_range(*min..=*max))),
                            ValueType::Float(max) => {
                                Ok(ValueType::Float(rng.gen_range((*min as f64)..=*max)))
                            }

                            _ => Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER)),
                        },
                        ValueType::Float(min) => match &args[1] {
                            ValueType::Int(max) => {
                                Ok(ValueType::Float(rng.gen_range(*min..=(*max as f64))))
                            }
                            ValueType::Float(max) => {
                                Ok(ValueType::Float(rng.gen_range(*min..=*max)))
                            }
                            _ => Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER)),
                        },
                        _ => Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER)),
                    }
                }
            }
            CommandType::Capitalize => {
                if args.len() != 1 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_ONE_ARG));
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
                        _ => Err(command_type.gen_err(ERROR_ARG_MUST_BE_TEXT)),
                    }
                }
            }
            CommandType::Upper => {
                if args.len() != 1 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_ONE_ARG));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => Ok(ValueType::Text(text.to_uppercase())),
                        _ => Err(command_type.gen_err(ERROR_ARG_MUST_BE_TEXT)),
                    }
                }
            }
            CommandType::Lower => {
                if args.len() != 1 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_ONE_ARG));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => Ok(ValueType::Text(text.to_lowercase())),
                        _ => Err(command_type.gen_err(ERROR_ARG_MUST_BE_TEXT)),
                    }
                }
            }
            CommandType::Repeat => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                } else {
                    match &args[0] {
                        ValueType::Int(value) => {
                            if *value > REPEAT_LIMIT.into() {
                                return Err(command_type.gen_err(&format!(
                                    "must not exceed more than {} repetitions",
                                    REPEAT_LIMIT
                                )));
                            }
                            for _i in 0..*value {
                                for arg in &args[1..args.len()] {
                                    if let ValueType::Command(command) = arg {
                                        self.eval_command(command.clone())?;
                                    } else {
                                        return Err(command_type
                                            .gen_err(ERROR_ARGS_AFTER_ARG_ONE_MUST_BE_COMMAND));
                                    };
                                }
                            }
                            return Ok(ValueType::None);
                        }
                        _ => {
                            return Err(command_type.gen_err(ERROR_ARG_ONE_MUST_BE_WHOLE_NUMBER));
                        }
                    }
                }
            }
            CommandType::Copy => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
                } else {
                    if let ValueType::Identifier(identifier) = &args[1] {
                        match &args[0] {
                            ValueType::Identifier(_) => {
                                Err(command_type.gen_err(ERROR_ARG_ONE_MUST_NOT_BE_IDENTIFIER))
                            }
                            ValueType::None => {
                                Err(command_type.gen_err(ERROR_ARG_ONE_MUST_NOT_BE_NONE))
                            }
                            _ => {
                                match self
                                    .vars
                                    .insert_var(identifier.to_string(), args[0].clone())
                                {
                                    Ok(_) => return Ok(ValueType::None),
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(command_type.gen_err(ERROR_ARG_TWO_MUST_BE_IDENTIFIER));
                    }
                }
            }
            CommandType::Paste => {
                if args.len() != 1 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_ONE_ARG));
                } else {
                    match &args[0] {
                        ValueType::Identifier(identifier) => match self.vars.get_var(identifier) {
                            Some(value) => Ok(value.clone()),
                            None => Err(command_type.gen_err(&format!(
                                "{} **{}**",
                                ERROR_UNKNOWN_IDENTIFIER, identifier
                            ))),
                        },
                        _ => Err(command_type.gen_err(ERROR_ARG_MUST_BE_IDENTIFIER)),
                    }
                }
            }
            CommandType::Print => {
                for arg in args {
                    let arg_string = arg.to_string();
                    if self.output.capacity().saturating_add(arg_string.capacity())
                        <= OUTPUT_BYTE_LIMIT
                    {
                        self.output.push_str(&arg_string);
                    } else {
                        return Err(format!(
                            "Output byte limit of {} bytes exceeded",
                            OUTPUT_BYTE_LIMIT
                        ));
                    }
                }

                Ok(ValueType::None)
            }
            CommandType::RemoveWhitespace => {
                if args.len() != 1 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_ONE_ARG));
                } else {
                    match &args[0] {
                        ValueType::Text(text) => {
                            Ok(ValueType::Text(text.split_whitespace().collect()))
                        }
                        _ => Err(command_type.gen_err(ERROR_ARG_MUST_BE_TEXT)),
                    }
                }
            }
            CommandType::Concatenate => {
                let mut concatenation = String::new();

                for arg in args {
                    concatenation.push_str(&arg.to_string());
                }

                Ok(ValueType::Text(concatenation))
            }
            CommandType::IfThen => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
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
                        _ => Err(command_type.gen_err(ERROR_ARG_ONE_MUST_BE_BOOL)),
                    }
                }
            }
            CommandType::IfThenElse => {
                if args.len() != 3 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_THREE_ARGS));
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
                        _ => Err(command_type.gen_err(ERROR_ARG_ONE_MUST_BE_BOOL)),
                    }
                }
            }
            CommandType::Not => {
                if args.len() != 1 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_ONE_ARG));
                } else {
                    match &args[0] {
                        ValueType::Bool(bool) => Ok(ValueType::Bool(!*bool)),
                        _ => Err(command_type.gen_err(ERROR_ARG_ONE_MUST_BE_BOOL)),
                    }
                }
            }
            CommandType::And => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                }
                let mut value: bool = true;
                for arg in args {
                    match arg {
                        ValueType::Bool(bool) => value = value & bool,
                        _ => return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_BOOL)),
                    }
                }
                Ok(ValueType::Bool(value))
            }
            CommandType::Or => {
                if args.len() < 2 {
                    return Err(command_type.gen_err(ERROR_TWO_OR_MORE_ARGS));
                }
                let mut value: bool = false;
                for arg in args {
                    match arg {
                        ValueType::Bool(bool) => value = value | bool,
                        _ => return Err(command_type.gen_err(ERROR_ARGS_MUST_BE_BOOL)),
                    }
                }
                Ok(ValueType::Bool(value))
            }
            CommandType::Eq => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
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
                        _ => Err(command_type.gen_err(&format!(
                            "Cannot compare {} with {}",
                            args[0].to_string(),
                            args[1].to_string()
                        ))),
                    }
                }
            }
            CommandType::Gt => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
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
                        _ => Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER)),
                    }
                }
            }
            CommandType::Lt => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
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
                        _ => Err(command_type.gen_err(ERROR_ARGS_MUST_BE_NUMBER)),
                    }
                }
            }
            CommandType::StartsWith => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
                } else {
                    match (&args[0], &args[1]) {
                        (ValueType::Text(value_a), ValueType::Text(value_b)) => {
                            Ok(ValueType::Bool(value_a.starts_with(value_b)))
                        }
                        _ => Err(command_type.gen_err(ERROR_ARGS_MUST_BE_TEXT)),
                    }
                }
            }
            CommandType::EndsWith => {
                if args.len() != 2 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_TWO_ARGS));
                } else {
                    match (&args[0], &args[1]) {
                        (ValueType::Text(value_a), ValueType::Text(value_b)) => {
                            Ok(ValueType::Bool(value_a.ends_with(value_b)))
                        }
                        _ => Err(command_type.gen_err(ERROR_ARGS_MUST_BE_TEXT)),
                    }
                }
            }
            CommandType::GetSub => {
                if args.len() != 1 {
                    return Err(command_type.gen_err(ERROR_EXACTLY_ONE_ARG));
                } else {
                    match &args[0] {
                        ValueType::Text(sub) => {
                            let output = self.interpolator.interp(
                                &(TEMPLATE_CARROT.to_string() + sub),
                                &|template| match self.db.get_random_subs(template) {
                                    Ok(sub) => Some(sub),
                                    Err(_) => None,
                                },
                            );

                            match output {
                                Ok(o) => Ok(ValueType::Text(o)),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        _ => Err(command_type.gen_err(ERROR_ARG_MUST_BE_TEXT)),
                    }
                }
            }
            CommandType::NewLine => {
                if args.len() != 0 {
                    Err(command_type.gen_err(ERROR_NO_ARGS))
                } else {
                    Ok(ValueType::Text("\n".to_string()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::interpreter::{parser::ValueType, Interpreter};

    #[test]
    fn interpret_new_line() {
        let code = "print(\"hello\", nl(), \"world\")";

        let mut interpreter = Interpreter::new();
        let output = interpreter.interpret(code).unwrap();

        assert_eq!(output, "hello\nworld");
    }

    #[test]
    fn interpret_get_sub() {
        let code = "print(get_sub(\"noun\"))";

        let mut interpreter = Interpreter::new();
        let output = interpreter.interpret(code).unwrap();

        dbg!(&output);
        assert_ne!(output, "noun");
        assert!(output.len() > 0);
    }

    #[test]
    fn interpret_repeat() {
        let code = "repeat(5, print(\"hello \"))";

        let mut interpreter = Interpreter::new();
        let output = interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::None);
        assert_eq!(output, "hello hello hello hello hello ");
    }

    #[test]
    fn interpret_if_then() {
        let code = "if_then(true, \"true\") if_then(false, \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::None);
    }

    #[test]
    fn interpret_if_then_else() {
        let code = "if_then_else(false, \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
    }

    #[test]
    fn interpret_not() {
        let code = "if_then_else(not(false), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_and() {
        let code = "if_then_else(and(true, false), \"true\", \"false\") if_then_else(and(true, true), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_or() {
        let code = "if_then_else(or(true, false), \"true\", \"false\") if_then_else(or(true, true), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_eq() {
        let code = "if_then_else(eq(1, 1), \"true\", \"false\") if_then_else(eq(1, 2), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("false".to_string()));
    }

    #[test]
    fn interpret_gt() {
        let code = "if_then_else(gt(1, 1), \"true\", \"false\") if_then_else(gt(2, 1), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_lt() {
        let code = "if_then_else(lt(1, 1), \"true\", \"false\") if_then_else(lt(1, 2), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("false".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("true".to_string()));
    }

    #[test]
    fn interpret_starts_with() {
        let code = "if_then_else(starts_with(\"apple\", \"a\"), \"true\", \"false\") if_then_else(starts_with(\"apple\", \"b\"), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("false".to_string()));
    }

    #[test]
    fn interpret_ends_with() {
        let code = "if_then_else(ends_with(\"apple\", \"e\"), \"true\", \"false\") if_then_else(ends_with(\"apple\", \"b\"), \"true\", \"false\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("true".to_string()));
        assert_eq!(interpreter.log[1], ValueType::Text("false".to_string()));
    }

    #[test]
    fn interpret_add() {
        let code = "add(5, 5) add(5.1, 5.2) add(1.0, 1.0) add(1.0, 1)";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Int(10));
        assert_eq!(interpreter.log[1], ValueType::Float(10.3));
        assert_eq!(interpreter.log[2], ValueType::Float(2.0));
        assert_eq!(interpreter.log[3], ValueType::Float(2.0));
    }

    #[test]
    fn interpret_subtract() {
        let code = "sub(-5, 5, -2) sub(4.2, 1.2) sub(1.0, 1.0) sub(1.0, 1)";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

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
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Int(10));
        assert_eq!(interpreter.log[1], ValueType::Float(10.0));
    }

    #[test]
    fn interpret_divide() {
        let code = "div(2, 3) div(3, 2) div (5.0, 2.5)";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

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
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("5text".to_string()));
    }

    #[test]
    fn interpret_capitalize() {
        let code = "capitalize(\"text\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("Text".to_string()));

        let code = "capitalize(\"t\")";
        interpreter.interpret_and_log(code).unwrap();
        assert_eq!(interpreter.log[1], ValueType::Text("T".to_string()));

        let code = "capitalize(\"\")";
        interpreter.interpret_and_log(code).unwrap();
        assert_eq!(interpreter.log[2], ValueType::Text("".to_string()));
    }

    #[test]
    fn interpret_upper() {
        let code = "upper(\"text\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("TEXT".to_string()));
    }

    #[test]
    fn interpret_lower() {
        let code = "lower(\"TEXT\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

        assert_eq!(interpreter.log[0], ValueType::Text("text".to_string()));
    }

    #[test]
    fn interpret_remove_whitespace() {
        let code = "remove_whitespace(\"text with whitespace removed\")";

        let mut interpreter = Interpreter::new();
        interpreter.interpret_and_log(code).unwrap();

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
        interpreter.interpret_and_log(code).unwrap();

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
        interpreter.interpret_and_log(code).unwrap();

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

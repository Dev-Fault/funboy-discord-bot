use crate::{fsl_interpreter::Interpreter, text_interpolator::TextInterpolator};

pub fn interp_input(
    input: &str,
    substitutor: &impl Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut interpolator = TextInterpolator::default();
    let mut fsl_interpreter = Interpreter::new();

    let output = interpolator.interp(input, substitutor);

    match output {
        Ok(output) => match fsl_interpreter.interpret_embedded_code(&output) {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        },
        Err(e) => Err(e.to_string()),
    }
}

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    fsl_interpreter::Interpreter, storage::template_database::FunboyDatabase,
    text_interpolator::TextInterpolator,
};

pub async fn interp_input(input: String, db: Arc<Mutex<FunboyDatabase>>) -> Result<String, String> {
    let mut interpolator = TextInterpolator::default();

    let fdb = db.lock().await;
    let output = interpolator.interp(&input, &|template| match fdb.get_random_subs(template) {
        Ok(sub) => Some(sub),
        Err(_) => None,
    });
    drop(fdb);

    let mut fsl_interpreter = Interpreter::new_with_db(db);
    match output {
        Ok(output) => match fsl_interpreter.interpret_embedded_code(&output).await {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        },
        Err(e) => Err(e.to_string()),
    }
}

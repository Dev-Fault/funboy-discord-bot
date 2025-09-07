use ollama_rs::{
    error::OllamaError,
    generation::completion::{request::GenerationRequest, GenerationResponse},
    models::{LocalModel, ModelInfo, ModelOptions},
    Ollama,
};

const DEFAULT_SYSTEM_PROMPT: &str = "";
const DEFAULT_TEMPLATE: &str = "{{ .Prompt }}";
const DEFAULT_MAX_PREDICT: u16 = 200;
const PARAMETER_NOT_SET_TEXT: &str = "Default";
pub const MAX_PREDICT: u16 = 2000;

pub struct OllamaParameters {
    pub temperature: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub top_k: Option<u32>,
    pub top_p: Option<f32>,
}

impl OllamaParameters {
    pub fn new() -> Self {
        Self {
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
        }
    }

    pub fn reset(&mut self) {
        self.temperature = None;
        self.repeat_penalty = None;
        self.top_k = None;
        self.top_p = None;
    }
}

pub struct OllamaConfig {
    system_prompt: String,
    template: String,
    output_limit: u16,
    parameters: OllamaParameters,
}

impl OllamaConfig {
    pub fn new() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            template: DEFAULT_TEMPLATE.to_string(),
            output_limit: DEFAULT_MAX_PREDICT,
            parameters: OllamaParameters::new(),
        }
    }
}

impl ToString for OllamaConfig {
    fn to_string(&self) -> String {
        format!("System Prompt: {}\nTemplate: {}\nOutput Limit: {}\nTemperature: {}\nRepeat Penalty: {}\nTop_k: {}\nTop_p: {}",
            self.system_prompt,
            self.template,
            self.output_limit,
            if self.parameters.temperature.is_none() {PARAMETER_NOT_SET_TEXT.to_string()} else {self.parameters.temperature.unwrap().to_string()},
            if self.parameters.repeat_penalty.is_none() {PARAMETER_NOT_SET_TEXT.to_string()} else {self.parameters.repeat_penalty.unwrap().to_string()},
            if self.parameters.top_k.is_none() {PARAMETER_NOT_SET_TEXT.to_string()} else {self.parameters.top_k.unwrap().to_string()},
            if self.parameters.top_p.is_none() {PARAMETER_NOT_SET_TEXT.to_string()} else {self.parameters.top_p.unwrap().to_string()},
        )
    }
}

pub struct OllamaGenerator {
    ollama: Ollama,
    current_model: Option<String>,
    config: OllamaConfig,
}

impl OllamaGenerator {
    pub fn new() -> Self {
        Self {
            ollama: Ollama::default(),
            current_model: None,
            config: OllamaConfig::new(),
        }
    }

    pub async fn get_models(&self) -> Result<Vec<LocalModel>, OllamaError> {
        self.ollama.list_local_models().await
    }

    pub async fn get_model_info(&self) -> Result<ModelInfo, OllamaError> {
        self.ollama
            .show_model_info(self.current_model.clone().unwrap_or("".to_string()))
            .await
    }

    pub fn get_config(&self) -> &OllamaConfig {
        &self.config
    }

    pub async fn get_current_model(&self) -> Option<String> {
        match &self.current_model {
            Some(name) => Some(name.to_string()),
            None => {
                let available_models = self.get_models().await;
                match available_models {
                    Ok(models) => Some(models[0].name.clone()),
                    Err(_) => None,
                }
            }
        }
    }

    pub fn set_current_model(&mut self, model: &str) {
        self.current_model = Some(model.to_string());
    }

    pub fn set_system_prompt(&mut self, prompt: &str) {
        self.config.system_prompt = prompt.to_string();
    }

    pub fn reset_system_prompt(&mut self) {
        self.config.system_prompt = DEFAULT_SYSTEM_PROMPT.to_string();
    }

    pub fn set_template(&mut self, template: &str) {
        self.config.template = template.to_string();
    }

    pub fn reset_template(&mut self) {
        self.config.template = DEFAULT_TEMPLATE.to_string();
    }

    pub fn set_output_limit(&mut self, limit: u16) -> bool {
        if limit > MAX_PREDICT {
            false
        } else {
            self.config.output_limit = limit;
            true
        }
    }

    pub fn set_parameters(&mut self, parameters: OllamaParameters) {
        self.config.parameters = parameters;
    }

    pub fn reset_parameters(&mut self) {
        self.config.parameters.reset();
    }

    pub fn set_temperature(&mut self, temperature: f32) {
        self.config.parameters.temperature = Some(temperature);
    }
    pub fn set_repeat_penalty(&mut self, repeat_penalty: f32) {
        self.config.parameters.repeat_penalty = Some(repeat_penalty);
    }
    pub fn set_top_k(&mut self, top_k: u32) {
        self.config.parameters.top_k = Some(top_k);
    }
    pub fn set_top_p(&mut self, top_p: f32) {
        self.config.parameters.top_p = Some(top_p);
    }

    fn generate_options(&self) -> ModelOptions {
        let mut options = ModelOptions::default();
        let parameters = &self.config.parameters;
        options = options.num_predict(self.config.output_limit.into());
        if let Some(temperature) = parameters.temperature {
            options = options.temperature(temperature);
        }
        if let Some(repeat_penalty) = parameters.repeat_penalty {
            options = options.repeat_penalty(repeat_penalty);
        }
        if let Some(top_k) = parameters.top_k {
            options = options.top_k(top_k);
        }
        if let Some(top_p) = parameters.top_p {
            options = options.top_p(top_p);
        }
        options
    }

    pub async fn generate(
        &self,
        prompt: &str,
        temperature_override: Option<f32>,
        model_override: Option<String>,
    ) -> Result<GenerationResponse, OllamaError> {
        let mut override_options = self.generate_options();
        if let Some(t) = temperature_override {
            override_options = override_options.temperature(t);
        }
        let model = match model_override {
            Some(model) => model,
            None => match &self.current_model {
                Some(name) => name.to_string(),
                None => {
                    let available_models = self.get_models().await;
                    match available_models {
                        Ok(models) => models[0].name.clone(),
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            },
        };

        let mut request = GenerationRequest::new(model, prompt).options(override_options);
        request = request.system(self.config.system_prompt.clone());
        request = request.template(self.config.template.clone());
        self.ollama.generate(request).await
    }
}

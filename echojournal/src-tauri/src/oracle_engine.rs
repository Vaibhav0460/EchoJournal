use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

pub struct OracleEngine {
    model: LlamaModel,
    backend: Arc<LlamaBackend>,
}

impl OracleEngine {
    pub fn new(model_path: &Path) -> Result<Self, String> {
        let backend = Arc::new(LlamaBackend::init().map_err(|e| e.to_string())?);
        let model_params = llama_cpp_2::model::params::LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|_| "Failed to load local model. Ensure .gguf exists in AppData/models/".to_string())?;

        Ok(Self { model, backend })
    }

    pub fn generate_response(&self, system_prompt: &str, user_query: &str) -> Result<String, String> {
        // Setup Context with NonZero safety
        let n_ctx = NonZeroU32::new(2048).unwrap();
        let ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));
        
        let mut context = self.model.new_context(&self.backend, ctx_params)
            .map_err(|e| e.to_string())?;
    
        let full_prompt = format!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n",
            system_prompt, user_query
        );
    
        let tokens: Vec<LlamaToken> = self.model
            .str_to_token(&full_prompt, AddBos::Always)
            .map_err(|e| e.to_string())?;
    
        let mut batch = LlamaBatch::new(2048, 1);
        for (i, token) in tokens.iter().enumerate() {
            let _ = batch.add(*token, i as i32, &[0], i == tokens.len() - 1);
        }
    
        context.decode(&mut batch).map_err(|e| e.to_string())?;
    
        let mut n_cur = tokens.len() as i32;
        let mut response_text = String::new();

        // Greedy sampling; keep sampler state in sync with the prompt.
        let mut sampler = LlamaSampler::greedy().with_tokens(tokens.iter());
        let mut decoder = encoding_rs::UTF_8.new_decoder();
    
        for _ in 0..500 {
            // Sample from the last decoded logits.
            let mut data_array = context.token_data_array();
            sampler.apply(&mut data_array);
            let token = data_array
                .selected_token()
                .ok_or_else(|| "Sampling failed to select a token".to_string())?;

            sampler.accept(token);

            if self.model.is_eog_token(token) || token == self.model.token_eos() {
                break;
            }

            let piece = self
                .model
                .token_to_piece(token, &mut decoder, false, None)
                .map_err(|e| e.to_string())?;
                
            response_text.push_str(&piece);

            // If the model is responding with a JSON object, stop as soon as we have a complete one.
            // This reduces the chance we run out of tokens and end up with truncated JSON.
            let trimmed = response_text.trim_end();
            if trimmed.starts_with('{')
                && trimmed.ends_with('}')
                && trimmed.contains("\"tag\"")
                && trimmed.contains("\"content\"")
            {
                break;
            }
    
            batch.clear();
            let _ = batch.add(token, n_cur, &[0], true);
            
            context.decode(&mut batch).map_err(|e| e.to_string())?;
            n_cur += 1;
        }
    
        Ok(response_text)
    }
}
//! LiteRT-LM C wrapper providing safe Rust interface

use std::ffi::{CStr, CString};
use serde::{Serialize, Deserialize};
use crate::error::{LlmError, LlmResult};
use crate::bindings::*;

/// Backend type for LiteRT-LM
#[derive(Debug, Clone)]
pub enum LiteRTBackend {
    Cpu,
    Gpu,
}

/// Safe wrapper around LiteRT-LM Engine
pub struct LiteRTEngine {
    inner: *mut LiteRtLmEngine,
    settings: *mut LiteRtLmEngineSettings,
}

unsafe impl Send for LiteRTEngine {}
unsafe impl Sync for LiteRTEngine {}

impl LiteRTEngine {
    /// Create new LiteRT engine with model
    pub fn new(model_path: &str, backend: LiteRTBackend) -> LlmResult<Self> {
        let model_path_cstr = CString::new(model_path)
            .map_err(|e| LlmError::BindingError(format!("Invalid model path: {}", e)))?;

        let backend_str = match backend {
            LiteRTBackend::Cpu => CString::new("cpu").unwrap(),
            LiteRTBackend::Gpu => CString::new("gpu").unwrap(),
        };

        // Create engine settings
        let settings = unsafe {
            litert_lm_engine_settings_create(model_path_cstr.as_ptr(), backend_str.as_ptr())
        };

        if settings.is_null() {
            return Err(LlmError::BindingError("Failed to create engine settings".to_string()));
        }

        // Create engine
        let engine = unsafe {
            litert_lm_engine_create(settings)
        };

        if engine.is_null() {
            unsafe { litert_lm_engine_settings_delete(settings); }
            return Err(LlmError::BindingError("Failed to create engine".to_string()));
        }

        Ok(LiteRTEngine {
            inner: engine,
            settings,
        })
    }

    /// Create new session
    pub fn create_session(&self) -> LlmResult<LiteRTSession> {
        LiteRTSession::new(self)
    }
}

impl Drop for LiteRTEngine {
    fn drop(&mut self) {
        unsafe {
            litert_lm_engine_delete(self.inner);
            litert_lm_engine_settings_delete(self.settings);
        }
    }
}

/// Safe wrapper around LiteRT-LM Session
pub struct LiteRTSession {
    inner: *mut LiteRtLmSession,
}

unsafe impl Send for LiteRTSession {}
unsafe impl Sync for LiteRTSession {}

impl LiteRTSession {
    fn new(engine: &LiteRTEngine) -> LlmResult<Self> {
        let session = unsafe {
            litert_lm_engine_create_session(engine.inner)
        };

        if session.is_null() {
            return Err(LlmError::BindingError("Failed to create session".to_string()));
        }

        Ok(LiteRTSession { inner: session })
    }

    /// Generate content from text prompt
    pub fn generate(&self, prompt: &str) -> LlmResult<String> {
        let prompt_cstr = CString::new(prompt)
            .map_err(|e| LlmError::BindingError(format!("Invalid prompt: {}", e)))?;

        // Create InputData for text
        let input_data = InputData {
            type_: InputDataType_kInputText,
            data: prompt_cstr.as_ptr() as *const std::os::raw::c_void,
            size: prompt.len(),
        };

        // Generate content
        let responses = unsafe {
            litert_lm_session_generate_content(self.inner, &input_data, 1)
        };

        if responses.is_null() {
            return Err(LlmError::BindingError("Failed to generate content".to_string()));
        }

        // Get response text
        let response_text = unsafe {
            let text_ptr = litert_lm_responses_get_response_text_at(responses, 0);
            if text_ptr.is_null() {
                litert_lm_responses_delete(responses);
                return Err(LlmError::BindingError("No response generated".to_string()));
            }
            let text = CStr::from_ptr(text_ptr).to_string_lossy().to_string();
            litert_lm_responses_delete(responses);
            text
        };

        Ok(response_text)
    }
}

impl Drop for LiteRTSession {
    fn drop(&mut self) {
        unsafe {
            litert_lm_session_delete(self.inner);
        }
    }
}

/// Tool definition for LiteRT-LM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Structured LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_types() {
        let cpu = LiteRTBackend::Cpu;
        let gpu = LiteRTBackend::Gpu;
        assert!(matches!(cpu, LiteRTBackend::Cpu));
        assert!(matches!(gpu, LiteRTBackend::Gpu));
    }
}

use futures::StreamExt;
use reqwest;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;


use crate::{session::{manager::SessionManager, message::Message}};

#[derive(Debug, Serialize, Deserialize)]
struct EventSteamDataChoice {
    delta: EventSteamDataDelta,
}
#[derive(Debug, Serialize, Deserialize)]
struct EventSteamDataDelta {
    content: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct EventSteamData {
    choices: Vec<EventSteamDataChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptTokensDetails {
    cached_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletion {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
    system_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    index: u32,
    message: Message,
    logprobs: Option<()>,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
    prompt_tokens_details: Option<PromptTokensDetails>,
    prompt_cache_hit_tokens: u32,
    prompt_cache_miss_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    messages: Vec<Message>,
    model: String,
    stream: bool,
}

#[derive(Debug)]
pub enum AlterAIError {
    RequestFailed(reqwest::Error),
    InvalidResponse(String),
}

impl fmt::Display for AlterAIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlterAIError::RequestFailed(err) => write!(f, "Request failed: {}", err),
            AlterAIError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}

impl Error for AlterAIError {}

impl From<serde_json::Error> for AlterAIError {
    fn from(err: serde_json::Error) -> Self {
        AlterAIError::InvalidResponse(format!("Failed to parse JSON: {}", err))
    }
}

pub async fn deepseek_client(session_manager: &mut SessionManager) -> Result<(), AlterAIError> {
    let client: reqwest::Client = reqwest::Client::new();

    let mut headers = header::HeaderMap::new();

    let auth = format!("Bearer {}", session_manager.config.default_model.api_key);

    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&auth.as_str()).unwrap(),
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("text/event-stream"),
    );

    if let Some(session) = session_manager.get_current_session() {

        let question = RequestBody {
            messages: session.messages.clone(),
            model: session_manager.config.default_model.model.to_string(),
            stream: true,
        };
    
        let response = client
            .post(session_manager.config.default_model.api_url.as_str())
            .headers(headers)
            .json(&question)
            .send()
            .await
            .map_err(AlterAIError::RequestFailed)?;
        if response.status().is_success() {
            let mut stream = response.bytes_stream();
            let mut content = String::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(AlterAIError::RequestFailed)?;
                let chunk_str = String::from_utf8_lossy(&chunk);
                if chunk_str.is_empty() {
                    continue;
                }
                for line in chunk_str.lines() {
                    let line = line.trim();
                    if line.starts_with("data:") {
                        let json_str = line.trim_start_matches("data:").trim();
                        if json_str == "[DONE]" {
                            break;
                        }
                        match serde_json::from_str::<EventSteamData>(&json_str) {
                            Ok(steam_text) => {
                                content += &steam_text.choices[0].delta.content;
                                print!("{}", steam_text.choices[0].delta.content);
                            }
                            Err(err) => {
                                eprintln!("Failed to parse chunk: {}", err);
                                eprintln!("Problematic chunk: {}", line);
                            }
                        }
                    } else if line.starts_with("[DONE]") {
                        break;
                    }
                }
            }
            if let Some(session) = session_manager.get_current_session() {
                session.add_message("assistant", &content);
            }
            return Ok(());
        } else {
            return Err(AlterAIError::InvalidResponse(format!(
                "Request failed with status: {}",
                response.status()
            )));
        }
    }
    Err(AlterAIError::InvalidResponse(format!(
        "not found current_session",
    )))
}

use std::collections::HashMap;
use std::fs::File;
use std::io::{self};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

use crate::markdown::parser::FileParser;
use crate::session::config::Config;
use crate::session::message::Message;

// 自定义错误类型
#[derive(Debug)]
pub enum SessionError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    SessionNotFound(String),
    InvalidSessionId,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionError::IoError(e) => write!(f, "IO错误: {}", e),
            SessionError::JsonError(e) => write!(f, "JSON错误: {}", e),
            SessionError::SessionNotFound(id) => write!(f, "会话未找到: {}", id),
            SessionError::InvalidSessionId => write!(f, "无效的会话ID"),
        }
    }
}

impl Error for SessionError {}

impl From<io::Error> for SessionError {
    fn from(err: io::Error) -> SessionError {
        SessionError::IoError(err)
    }
}

impl From<serde_json::Error> for SessionError {
    fn from(err: serde_json::Error) -> SessionError {
        SessionError::JsonError(err)
    }
}

// 会话结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub messages: Vec<Message>,
}

impl Session {
    fn new(title: &str) -> Self {
        let now = Utc::now();
        Session {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            created_at: now,
            last_accessed: now,
            messages: vec![Message {
                role: "system".to_string(),
                content: "现在你是一个心灵使者 ， 不管用户说什么，你都要是用户心灵舒畅".to_string(),
                timestamp: now,
            }]
        }
    }
    
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        });
        self.last_accessed = Utc::now();
        
        // 如果标题为空，使用第一条用户消息作为标题
        if self.title.is_empty() && role == "user" {
            let preview = if content.len() > 20 {
                format!("{}...", &content[..20])
            } else {
                content.to_string()
            };
            self.title = preview;
        }
    }
    
    pub fn update_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.last_accessed = Utc::now();
    }
}


// 会话管理器
pub struct SessionManager {
    pub sessions: HashMap<String, Session>,
    pub current_session_id: Option<String>,
    pub config: Config,
    pub config_path: PathBuf,
}

impl SessionManager {
    pub fn new(config_path: PathBuf) -> Result<Self, SessionError> {
        let config: Config = if config_path.exists() {
            let config_file = File::open(&config_path)?;
            serde_json::from_reader(config_file)?
        } else {
            Config::default()
        };

        if !config_path.exists() {
            let default_config = Config::default();
            let config_file = File::create(&config_path)?;
            serde_json::to_writer_pretty(config_file, &default_config)?;
        }
        
        Ok(SessionManager {
            sessions: HashMap::new(),
            current_session_id: None,
            config,
            config_path,
        })
    }
    
    pub fn create_session(&mut self, title: &str) -> &str {
        // 清理旧会话
        if self.sessions.len() >= self.config.max_sessions {
            self.cleanup_old_sessions();
        }
        
        let session = Session::new(title);
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.current_session_id = Some(id.clone());
        &self.sessions[&id].id
    }
    
    pub fn switch_session(&mut self, session_id: &str) -> Result<(), SessionError> {
        if self.sessions.contains_key(session_id) {
            self.current_session_id = Some(session_id.to_string());
            Ok(())
        } else {
            Err(SessionError::SessionNotFound(session_id.to_string()))
        }
    }
    
    pub fn get_current_session(&mut self) -> Option<&mut Session> {
        if let Some(ref id) = self.current_session_id {
            self.sessions.get_mut(id)
        } else {
            None
        }
    }
    
    pub fn list_sessions(&self) -> Vec<&Session> {
        let mut sessions: Vec<&Session> = self.sessions.values().collect();
        
        // 按最后访问时间排序，最近的在前
        sessions.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        
        sessions
    }
    
    pub fn cleanup_old_sessions(&mut self) {
        if self.sessions.len() <= self.config.max_sessions {
            return;
        }
        
        let mut sessions_vec: Vec<&Session> = self.sessions.values().collect();
        sessions_vec.sort_by(|a, b| a.last_accessed.cmp(&b.last_accessed));
        
        // 移除最旧的，直到满足最大会话数限制
        for session in sessions_vec {
            if self.sessions.len() <= self.config.max_sessions {
                break;
            }
            
            // 不要移除当前会话
            if Some(&session.id) != self.current_session_id.as_ref() {
                // self.sessions.remove(&session.id);
            }
        }
    }
    
    pub fn remove_session(&mut self, session_id: &str) -> Result<(), SessionError> {
        // 不能移除当前会话
        if Some(session_id) == self.current_session_id.as_deref() {
            self.current_session_id = None;
        }
        
        if self.sessions.remove(session_id).is_some() {
            Ok(())
        } else {
            Err(SessionError::SessionNotFound(session_id.to_string()))
        }
    }
    
    pub fn rename_session(&mut self, session_id: &str, new_title: &str) -> Result<(), SessionError> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.update_title(new_title);
            Ok(())
        } else {
            Err(SessionError::SessionNotFound(session_id.to_string()))
        }
    }
    
    pub fn save_config(&self) -> Result<(), SessionError> {
        let config_file = File::create(&self.config_path)?;
        serde_json::to_writer_pretty(config_file, &self.config)?;
        Ok(())
    }
    
    pub fn save_sessions(&self, path: &PathBuf) -> Result<(), SessionError> {
        let session_file = File::create(path)?;
        serde_json::to_writer_pretty(session_file, &self.sessions)?;
        Ok(())
    }

    pub async fn generate_session_file(&mut self, session_id: Option<&str>) -> Result<(), SessionError> {
        let mut file_parser = FileParser::new();
        if let Some(id) = session_id {
            if let Some(session) = &self.sessions.get(&id.to_string()) {
                if let Some(last_message) = session.messages.last() {
                    let _ = file_parser.init(last_message.content.clone()).await;
                }
            } else {
                return Err(SessionError::SessionNotFound(id.to_string()));
            }
        } else if let Some(session) = &self.get_current_session() {
            if let Some(last_message) = session.messages.last() {
                let _ = file_parser.init(last_message.content.clone()).await;
            }
        } else {
            return Err(SessionError::InvalidSessionId);
        }
        Ok(())
    }
    
    pub fn load_sessions(&mut self, path: &PathBuf) -> Result<(), SessionError> {
        if path.exists() {
            let session_file = File::open(path)?;
            self.sessions = serde_json::from_reader(session_file)?;
            Ok(())
        } else {
            Err(SessionError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "会话文件不存在",
            )))
        }
    }
}

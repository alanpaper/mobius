use std::io::{self, Write};
use std::path::PathBuf;
use std::error::Error;


use crate::cli::alter::handle_command;
use crate::models::model::generate_response;
use crate::session::manager::SessionManager;

pub async fn main_loop(
    session_manager: &mut SessionManager, 
    sessions_path: &PathBuf
) -> Result<(), Box<dyn Error>> {

    let session_id = session_manager.current_session_id.as_ref()
        .ok_or("没有当前会话")?.clone();
    
    let session_title = session_manager.sessions.get(&session_id)
        .map(|s| s.title.clone())
        .unwrap_or_else(|| "未知会话".to_string());
    
    println!("会话: {} [ID: {}]", session_title, &session_id[..8]);
    println!("输入 /help 查看可用命令");
    
    // let mut last_save = Utc::now();
    
    loop {
        print!("\n>: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.starts_with('/') {
            if handle_command(&input[1..], session_manager, sessions_path)? {
                break;
            }
            continue;
        }
        
        if let Some(session) = session_manager.get_current_session() {
            session.add_message("user", input);
        }
        
        let _  = generate_response(session_manager).await?;
    }
    
    if session_manager.config.auto_save {
        session_manager.save_sessions(sessions_path)?;
    }
    
    Ok(())
}

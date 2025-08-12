use crate::{models::deepseek::deepseek_client, session::manager::SessionManager};

pub async fn generate_response(session_manager: &mut SessionManager) -> Result<(), anyhow::Error> {
   let _ = deepseek_client(session_manager).await?;
   Ok(())
}
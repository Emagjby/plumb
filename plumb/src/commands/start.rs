use thiserror::Error;

use crate::workspace::{
    WorkspaceError, ensure_no_active_session, ensure_plumb_dir, initialize_session,
    resolve_workspace_root,
};

#[derive(Debug, Error)]
pub enum StartError {
    #[error("{0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_start(name: Option<String>) -> Result<(), StartError> {
    let cwd = std::env::current_dir().map_err(|e| StartError::UnknownError(e.to_string()))?;

    let root = resolve_workspace_root(&cwd)?;
    ensure_plumb_dir(&root)?;

    ensure_no_active_session(&root)?;
    initialize_session(&root, name.as_deref().unwrap_or(""))?;

    Ok(())
}

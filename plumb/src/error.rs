use thiserror::Error;

use crate::{commands::start::PlumbStartError, workspace::WorkspaceError};

#[derive(Error, Debug)]
pub enum PlumbError {
    #[error("start error: {0}")]
    StartError(#[from] PlumbStartError),
    #[error("workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),
}

use thiserror::Error;

use crate::{
    commands::start::PlumbStartError, fs::InputError, store::items::StoreError,
    workspace::WorkspaceError,
};

#[derive(Error, Debug)]
pub enum PlumbError {
    #[error("store error: {0}")]
    StoreError(#[from] StoreError),
    #[error("start error: {0}")]
    StartError(#[from] PlumbStartError),
    #[error("workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("input error: {0}")]
    InputError(#[from] InputError),
}

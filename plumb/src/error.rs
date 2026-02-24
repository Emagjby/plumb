use thiserror::Error;

use crate::{
    commands::{add::AddError, start::PlumbStartError, status::StatusError},
    fs::InputError,
    store::items::StoreError,
    workspace::WorkspaceError,
};

#[derive(Error, Debug)]
pub enum PlumbError {
    #[error("status error: {0}")]
    StatusError(#[from] StatusError),
    #[error("add error: {0}")]
    AddError(#[from] AddError),
    #[error("store error: {0}")]
    StoreError(#[from] StoreError),
    #[error("start error: {0}")]
    StartError(#[from] PlumbStartError),
    #[error("workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("input error: {0}")]
    InputError(#[from] InputError),
}

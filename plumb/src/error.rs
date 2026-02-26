use thiserror::Error;

use crate::{
    commands::{
        add::AddError, diff::DiffError, done::DoneError, go::GoError, next::NextError, rm::RmError,
        start::StartError, status::StatusError,
    },
    fs::InputError,
    store::items::StoreError,
    workspace::WorkspaceError,
};

#[derive(Error, Debug)]
pub enum PlumbError {
    #[error("done error: {0}")]
    DoneError(#[from] DoneError),
    #[error("next error: {0}")]
    NextError(#[from] NextError),
    #[error("diff error: {0}")]
    DiffError(#[from] DiffError),
    #[error("go error: {0}")]
    GoError(#[from] GoError),
    #[error("rm error: {0}")]
    RmError(#[from] RmError),
    #[error("status error: {0}")]
    StatusError(#[from] StatusError),
    #[error("add error: {0}")]
    AddError(#[from] AddError),
    #[error("store error: {0}")]
    StoreError(#[from] StoreError),
    #[error("start error: {0}")]
    StartError(#[from] StartError),
    #[error("workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("input error: {0}")]
    InputError(#[from] InputError),
}

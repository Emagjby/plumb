use std::fmt::{self, Display, Formatter};

use crate::{
    commands::{
        add::AddError, diff::DiffError, done::DoneError, finish::FinishError, go::GoError,
        next::NextError, restore::RestoreError, rm::RmError, start::StartError,
        status::StatusError,
    },
    diagnostic::Diagnostic,
    fs::{FsError, InputError},
    helpers::HelperError,
    store::items::StoreError,
    workspace::WorkspaceError,
};

#[derive(Debug)]
pub enum PlumbError {
    RestoreError(RestoreError),
    FinishError(FinishError),
    DoneError(DoneError),
    NextError(NextError),
    DiffError(DiffError),
    GoError(GoError),
    RmError(RmError),
    StatusError(StatusError),
    AddError(AddError),
    StoreError(StoreError),
    StartError(StartError),
    WorkspaceError(WorkspaceError),
    InputError(InputError),
}

impl PlumbError {
    pub fn diagnostic(&self) -> Diagnostic {
        match self {
            PlumbError::RestoreError(err) => map_restore_error(err),
            PlumbError::FinishError(err) => map_finish_error(err),
            PlumbError::DoneError(err) => map_done_error(err),
            PlumbError::NextError(err) => map_next_error(err),
            PlumbError::DiffError(err) => map_diff_error(err),
            PlumbError::GoError(err) => map_go_error(err),
            PlumbError::RmError(err) => map_rm_error(err),
            PlumbError::StatusError(err) => map_status_error(err),
            PlumbError::AddError(err) => map_add_error(err),
            PlumbError::StoreError(err) => map_store_error("plumb", err),
            PlumbError::StartError(err) => map_start_error(err),
            PlumbError::WorkspaceError(err) => map_workspace_error("plumb", err),
            PlumbError::InputError(err) => map_input_error("plumb", err),
        }
    }
}

macro_rules! impl_from {
    ($variant:ident, $t:ty) => {
        impl From<$t> for PlumbError {
            fn from(value: $t) -> Self {
                PlumbError::$variant(value)
            }
        }
    };
}

impl_from!(RestoreError, RestoreError);
impl_from!(FinishError, FinishError);
impl_from!(DoneError, DoneError);
impl_from!(NextError, NextError);
impl_from!(DiffError, DiffError);
impl_from!(GoError, GoError);
impl_from!(RmError, RmError);
impl_from!(StatusError, StatusError);
impl_from!(AddError, AddError);
impl_from!(StoreError, StoreError);
impl_from!(StartError, StartError);
impl_from!(WorkspaceError, WorkspaceError);
impl_from!(InputError, InputError);

impl Display for PlumbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostic())
    }
}

impl std::error::Error for PlumbError {}

fn map_start_error(err: &StartError) -> Diagnostic {
    match err {
        StartError::WorkspaceError(inner) => map_workspace_error("start", inner),
        StartError::UnknownError(msg) => {
            base("PLB-WSP-001", "failed to resolve workspace root", "start")
                .with_hint("run command inside your workspace")
                .with_cause(msg)
        }
    }
}

fn map_add_error(err: &AddError) -> Diagnostic {
    match err {
        AddError::StoreError(inner) => map_store_error("add", inner),
        AddError::FileAlreadyInQueue(_) => base("PLB-ITM-006", "item already in queue", "add")
            .with_hint("choose a different path or remove the existing item first"),
        AddError::InputError(inner) => map_input_error("add", inner),
        AddError::NoActiveSession(_) => no_active_session("add"),
        AddError::UnknownError(msg) => {
            if contains_ci(msg, "path is not a directory") {
                base("PLB-WSP-002", "invalid input path", "add")
                    .with_hint("use `plumb add -f <folder>` with an existing directory")
                    .with_cause(msg)
            } else {
                base("PLB-IO-001", "filesystem operation failed", "add")
                    .with_hint("check path validity and permissions, then retry")
                    .with_cause(msg)
            }
        }
    }
}

fn map_rm_error(err: &RmError) -> Diagnostic {
    match err {
        RmError::StoreError(inner) => map_store_error("rm", inner),
        RmError::ItemInProgress(_) => base("PLB-ITM-004", "cannot remove item in progress", "rm")
            .with_hint("run `plumb done` or `plumb restore` first"),
        RmError::NoActiveSession(_) => no_active_session("rm"),
        RmError::FileNotInQueue(_) => target_not_found("rm"),
        RmError::HelperError(inner) => map_helper_error("rm", inner),
        RmError::UnknownError(msg) => base("PLB-IO-001", "filesystem operation failed", "rm")
            .with_hint("check permissions and retry")
            .with_cause(msg),
    }
}

fn map_status_error(err: &StatusError) -> Diagnostic {
    match err {
        StatusError::StoreError(inner) => map_store_error("status", inner),
        StatusError::NoActiveSession(_) => no_active_session("status"),
        StatusError::UnknownError(msg) => {
            base("PLB-WSP-001", "failed to resolve workspace root", "status")
                .with_hint("run command inside your workspace")
                .with_cause(msg)
        }
    }
}

fn map_go_error(err: &GoError) -> Diagnostic {
    match err {
        GoError::AlreadyDone(_) => base("PLB-ITM-002", "item is already done", "go")
            .with_hint("choose an item in `todo` or `in_progress` state"),
        GoError::AlreadyInProgress(_) => base("PLB-ITM-003", "item is already in progress", "go")
            .with_hint("continue working on the item or run `plumb done`"),
        GoError::NoActiveSession(_) => no_active_session("go"),
        GoError::FileNotInQueue(_) => target_not_found("go"),
        GoError::EditorError(msg) => {
            if contains_ci(msg, "editor exited with status") {
                base("PLB-EDT-002", "editor exited with failure", "go")
                    .with_hint("check your `EDITOR` command and retry")
                    .with_cause(msg)
            } else {
                base("PLB-EDT-001", "failed to launch editor", "go")
                    .with_hint("set `EDITOR` to a valid executable")
                    .with_cause(msg)
            }
        }
        GoError::BaselineCaptureError(msg) => {
            if contains_ci(msg, "file does not exist") {
                base("PLB-SNP-003", "baseline source file does not exist", "go")
                    .with_hint("create the file on disk or choose a different item")
            } else if contains_ci(msg, "cannot capture baseline for a folder") {
                base("PLB-SNP-004", "baseline source is a directory", "go")
                    .with_hint("provide a file target instead of a folder")
            } else if contains_ci(msg, "failed to read file") {
                base("PLB-SNP-005", "failed to read baseline source file", "go")
                    .with_hint("check file permissions and retry")
                    .with_cause(msg)
            } else {
                base("PLB-IO-001", "filesystem operation failed", "go")
                    .with_hint("check path validity and permissions, then retry")
                    .with_cause(msg)
            }
        }
        GoError::HelperError(inner) => map_helper_error("go", inner),
        GoError::StoreError(inner) => map_store_error("go", inner),
        GoError::FsError(inner) => map_fs_error("go", inner),
        GoError::UnknownError(msg) => base("PLB-INT-001", "internal invariant violation", "go")
            .with_hint("re-run the command; if it persists, report this bug")
            .with_cause(msg),
    }
}

fn map_diff_error(err: &DiffError) -> Diagnostic {
    match err {
        DiffError::FileReadError(msg) => {
            if contains_ci(msg, "no baseline snapshot") {
                base("PLB-SNP-001", "baseline snapshot required", "diff")
                    .with_hint("run `plumb go <id|path>` first")
            } else if contains_ci(msg, "baseline snapshot not found") {
                base("PLB-SNP-002", "baseline snapshot missing on disk", "diff")
                    .with_hint("rerun `plumb go <id|path>` or inspect session snapshots")
            } else if contains_ci(msg, "failed to read baseline snapshot") {
                base("PLB-IO-001", "failed to read baseline snapshot", "diff")
                    .with_hint("verify snapshot file permissions and retry")
                    .with_cause(msg)
            } else {
                base("PLB-IO-001", "filesystem operation failed", "diff")
                    .with_hint("check file permissions and retry")
                    .with_cause(msg)
            }
        }
        DiffError::FileWriteError(msg) => base("PLB-IO-001", "filesystem operation failed", "diff")
            .with_hint("retry the command")
            .with_cause(msg),
        DiffError::DiffComputationError(msg) => {
            base("PLB-INT-001", "failed to compute diff", "diff")
                .with_hint("re-run the command; if it persists, report this bug")
                .with_cause(msg)
        }
        DiffError::NoActiveSession(_) => no_active_session("diff"),
        DiffError::FileNotInQueue(_) => target_not_found("diff"),
        DiffError::HelperError(inner) => map_helper_error("diff", inner),
        DiffError::StoreError(inner) => map_store_error("diff", inner),
        DiffError::UnknownError(msg) => base("PLB-INT-001", "internal invariant violation", "diff")
            .with_hint("re-run the command; if it persists, report this bug")
            .with_cause(msg),
    }
}

fn map_done_error(err: &DoneError) -> Diagnostic {
    match err {
        DoneError::NoActiveSession(_) => no_active_session("done"),
        DoneError::FileNotInQueue(_) => target_not_found("done"),
        DoneError::HelperError(inner) => map_helper_error("done", inner),
        DoneError::StoreError(inner) => map_store_error("done", inner),
        DoneError::WorkspaceError(inner) => map_workspace_error("done", inner),
        DoneError::UnknownError(msg) => {
            if contains_ci(msg, "not 'in progress'") {
                base("PLB-ITM-003", "item is not in_progress", "done")
                    .with_hint("run `plumb go <id|path>` before `plumb done`")
            } else {
                base("PLB-INT-001", "internal invariant violation", "done")
                    .with_hint("re-run the command; if it persists, report this bug")
                    .with_cause(msg)
            }
        }
    }
}

fn map_next_error(err: &NextError) -> Diagnostic {
    match err {
        NextError::NoTodoInQueue(_) => {
            base("PLB-ITM-005", "no 'To Do' items found in the queue", "next")
                .with_hint("add more items with `plumb add <path>`")
        }
        NextError::NoActiveSession(_) => no_active_session("next"),
        NextError::WorkspaceError(inner) => map_workspace_error("next", inner),
        NextError::UnknownError(msg) => base("PLB-INT-001", "internal invariant violation", "next")
            .with_hint("re-run the command; if it persists, report this bug")
            .with_cause(msg),
    }
}

fn map_restore_error(err: &RestoreError) -> Diagnostic {
    match err {
        RestoreError::FileReadError(msg) => {
            if contains_ci(msg, "no baseline snapshot") {
                base("PLB-SNP-001", "baseline snapshot required", "restore")
                    .with_hint("run `plumb go <id|path>` first")
            } else if contains_ci(msg, "baseline snapshot not found") {
                base(
                    "PLB-SNP-002",
                    "baseline snapshot missing on disk",
                    "restore",
                )
                .with_hint("rerun `plumb go <id|path>` or inspect session snapshots")
            } else if contains_ci(msg, "file does not exist") {
                base(
                    "PLB-SNP-006",
                    "restore destination file does not exist",
                    "restore",
                )
                .with_hint("recreate the file or skip restore for this item")
            } else if contains_ci(msg, "cannot restore a folder") {
                base(
                    "PLB-SNP-007",
                    "restore destination is a directory",
                    "restore",
                )
                .with_hint("provide a file target instead of a folder")
            } else {
                base("PLB-IO-001", "filesystem operation failed", "restore")
                    .with_hint("check file state and permissions, then retry")
                    .with_cause(msg)
            }
        }
        RestoreError::FileWriteError(msg) => {
            if contains_any_ci(msg, &["cannot write to file", "permission denied"]) {
                base(
                    "PLB-SNP-008",
                    "restore destination is not writable",
                    "restore",
                )
                .with_hint("fix file permissions and retry")
                .with_cause(msg)
            } else {
                base("PLB-IO-001", "filesystem operation failed", "restore")
                    .with_hint("check file state and permissions, then retry")
                    .with_cause(msg)
            }
        }
        RestoreError::NoActiveSession(_) => no_active_session("restore"),
        RestoreError::FileNotInQueue(_) => target_not_found("restore"),
        RestoreError::HelperError(inner) => map_helper_error("restore", inner),
        RestoreError::StoreError(inner) => map_store_error("restore", inner),
        RestoreError::WorkspaceError(inner) => map_workspace_error("restore", inner),
        RestoreError::UnknownError(msg) => base("PLB-IO-003", "terminal I/O failed", "restore")
            .with_hint("retry in an interactive shell")
            .with_cause(msg),
    }
}

fn map_finish_error(err: &FinishError) -> Diagnostic {
    match err {
        FinishError::NoActiveSession(_) => no_active_session("finish"),
        FinishError::StoreError(inner) => map_store_error("finish", inner),
        FinishError::WorkspaceError(inner) => map_workspace_error("finish", inner),
        FinishError::UnknownError(msg) => {
            if contains_ci(msg, "in progress") {
                base(
                    "PLB-SES-004",
                    "cannot finish while items are in progress",
                    "finish",
                )
                .with_hint("run `plumb done <id|path>` first")
            } else {
                base("PLB-INT-001", "internal invariant violation", "finish")
                    .with_hint("re-run the command; if it persists, report this bug")
                    .with_cause(msg)
            }
        }
    }
}

fn map_workspace_error(command: &str, err: &WorkspaceError) -> Diagnostic {
    match err {
        WorkspaceError::SessionAlreadyActive { root, session_id } => {
            base("PLB-SES-002", "active session already exists", command)
                .with_context("workspace", root.display().to_string())
                .with_context("session_id", session_id.clone())
                .with_hint("run `plumb finish` first")
        }
        WorkspaceError::UnknownError(msg) => {
            if contains_any_ci(
                msg,
                &["corrupted session id", "expected 'active' to be a file"],
            ) {
                base("PLB-SES-003", "active session pointer is invalid", command)
                    .with_hint("inspect `.plumb/active` and `.plumb/sessions`")
                    .with_cause(msg)
            } else if contains_ci(msg, "failed to encode") {
                base("PLB-STO-004", "failed to encode state file", command)
                    .with_hint("retry and report if the problem persists")
                    .with_cause(msg)
            } else {
                base("PLB-WSP-004", "workspace layout is invalid", command)
                    .with_hint("repair your `.plumb` directory structure")
                    .with_cause(msg)
            }
        }
    }
}

fn map_store_error(command: &str, err: &StoreError) -> Diagnostic {
    match err {
        StoreError::NoActiveSession => no_active_session(command),
        StoreError::ResolveWorkspaceRootError(msg) => {
            base("PLB-WSP-001", "failed to resolve workspace root", command)
                .with_hint("run command inside your workspace")
                .with_cause(msg)
        }
        StoreError::ReadError(msg) => {
            if contains_any_ci(msg, &["decode scb", "failed to decode", "invalid scb"]) {
                base("PLB-STO-002", "failed to decode state file", command)
                    .with_hint("state file is corrupted or not valid SCB")
                    .with_cause(msg)
            } else if contains_any_ci(
                msg,
                &[
                    "expected a list",
                    "expected each item to be a map",
                    "missing or invalid",
                    "invalid state",
                    "expected a map",
                    "must be exactly 16 bytes",
                    "invalid session_id bytes",
                ],
            ) {
                base("PLB-STO-003", "state file has invalid schema", command)
                    .with_hint("repair the state file or recreate the session")
                    .with_cause(msg)
            } else {
                base("PLB-STO-001", "failed to read state file", command)
                    .with_hint("verify file exists and is readable")
                    .with_cause(msg)
            }
        }
        StoreError::WriteError(msg) => {
            if contains_ci(msg, "encode") {
                base("PLB-STO-004", "failed to encode state file", command)
                    .with_hint("retry and report if the problem persists")
                    .with_cause(msg)
            } else {
                base("PLB-STO-005", "failed to write state file", command)
                    .with_hint("check permissions and disk space, then retry")
                    .with_cause(msg)
            }
        }
    }
}

fn map_input_error(command: &str, err: &InputError) -> Diagnostic {
    match err {
        InputError::InvalidPath(path) => base("PLB-WSP-002", "invalid input path", command)
            .with_context("path", path.clone())
            .with_hint("provide a valid path inside the workspace"),
        InputError::EscapesRoot(path) => {
            base("PLB-WSP-003", "path escapes workspace root", command)
                .with_context("path", path.clone())
                .with_hint("use a path inside the workspace")
        }
    }
}

fn map_helper_error(command: &str, err: &HelperError) -> Diagnostic {
    match err {
        HelperError::PathNormalizationError(msg) => {
            if contains_ci(msg, "escapes workspace root") {
                base("PLB-WSP-003", "path escapes workspace root", command)
                    .with_hint("use a path inside the workspace")
                    .with_cause(msg)
            } else {
                base("PLB-WSP-002", "invalid input path", command)
                    .with_hint("provide a valid path")
                    .with_cause(msg)
            }
        }
        HelperError::FileNotInQueue(_) => target_not_found(command),
        HelperError::BaselineReadError(msg) => {
            if contains_ci(msg, "not found") {
                base("PLB-SNP-002", "baseline snapshot missing on disk", command)
                    .with_hint("rerun `plumb go <id|path>` or inspect session snapshots")
            } else {
                base("PLB-IO-001", "failed to read baseline snapshot", command)
                    .with_hint("check file permissions and retry")
                    .with_cause(msg)
            }
        }
        HelperError::UnknownError(msg) => {
            base("PLB-INT-001", "internal invariant violation", command)
                .with_hint("re-run the command; if it persists, report this bug")
                .with_cause(msg)
        }
    }
}

fn map_fs_error(command: &str, err: &FsError) -> Diagnostic {
    match err {
        FsError::IoError(io_err) => {
            if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                base("PLB-IO-002", "permission denied", command)
                    .with_hint("fix permissions and retry")
                    .with_cause(io_err.to_string())
            } else {
                base("PLB-IO-001", "filesystem operation failed", command)
                    .with_hint("check file permissions and path validity, then retry")
                    .with_cause(io_err.to_string())
            }
        }
        FsError::AtomicWriteError(msg) => {
            base("PLB-STO-005", "failed to write state file", command)
                .with_hint("check permissions and disk space, then retry")
                .with_cause(msg)
        }
    }
}

fn base(code: &'static str, summary: &'static str, command: &str) -> Diagnostic {
    Diagnostic::error(code, summary).with_command(command_label(command))
}

fn command_label(command: &str) -> String {
    if command == "plumb" {
        "plumb".to_string()
    } else {
        format!("plumb {command}")
    }
}

fn no_active_session(command: &str) -> Diagnostic {
    base("PLB-SES-001", "no active session found", command)
        .with_hint("use `plumb start` to start a session")
}

fn target_not_found(command: &str) -> Diagnostic {
    base("PLB-ITM-001", "target item not found in queue", command)
        .with_hint("run `plumb status` to list valid item IDs and paths")
}

fn contains_ci(text: &str, needle: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn contains_any_ci(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| contains_ci(text, needle))
}

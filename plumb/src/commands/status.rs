use thiserror::Error;

use crate::{
    store::items::{active_session_id, load_items, Item, State, StoreError},
    workspace::resolve_workspace_root,
};

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("{0}")]
    StoreError(#[from] StoreError),
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_status() -> Result<(), StatusError> {
    let cwd = std::env::current_dir().map_err(|e| StatusError::UnknownError(e.to_string()))?;
    let root =
        resolve_workspace_root(&cwd).map_err(|e| StatusError::UnknownError(e.to_string()))?;

    let session_id = active_session_id(&root).map_err(|_| {
        StatusError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    let items = load_items(&root)?;

    print_status(&items, &session_id);

    Ok(())
}

fn print_status(items: &[Item], session_id: &str) {
    println!("Session [{}] Status:", session_id);
    print_compact(items);

    println!("\nQueue:");
    print_queue(items);
}

fn print_compact(items: &[Item]) {
    let mut todo_count = 0;
    let mut in_progress_count = 0;
    let mut done_count = 0;

    for item in items {
        match item.state {
            State::Todo => todo_count += 1,
            State::InProgress => in_progress_count += 1,
            State::Done => done_count += 1,
        }
    }

    println!("  {} item(s) [TODO]", todo_count);
    println!("  {} item(s) [IN_PROGRESS]", in_progress_count);
    println!("  {} item(s) [DONE]", done_count);
}

fn print_queue(item: &[Item]) {
    for item in item {
        println!("  [{:?}] {:?} - {:?}", item.id, item.rel_path, item.state);
    }
}

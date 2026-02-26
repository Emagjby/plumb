use std::fmt::{self, Display, Formatter};

use crate::verbosity::is_verbose;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLevel {
    Ok,
    Info,
    Warn,
    Prompt,
}

#[derive(Debug, Clone)]
pub struct OutputMessage {
    level: OutputLevel,
    code: &'static str,
    summary: String,
    command: Option<String>,
    context: Vec<(String, String)>,
    note: Option<String>,
    action: Option<String>,
}

impl OutputMessage {
    pub fn ok(code: &'static str, summary: impl Into<String>) -> Self {
        Self::new(OutputLevel::Ok, code, summary)
    }

    pub fn info(code: &'static str, summary: impl Into<String>) -> Self {
        Self::new(OutputLevel::Info, code, summary)
    }

    pub fn warn(code: &'static str, summary: impl Into<String>) -> Self {
        Self::new(OutputLevel::Warn, code, summary)
    }

    pub fn prompt(code: &'static str, summary: impl Into<String>) -> Self {
        Self::new(OutputLevel::Prompt, code, summary)
    }

    fn new(level: OutputLevel, code: &'static str, summary: impl Into<String>) -> Self {
        Self {
            level,
            code,
            summary: summary.into(),
            command: None,
            context: Vec::new(),
            note: None,
            action: None,
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }
}

impl Display for OutputMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let level = match self.level {
            OutputLevel::Ok => "ok",
            OutputLevel::Info => "info",
            OutputLevel::Warn => "warn",
            OutputLevel::Prompt => "prompt",
        };

        if !is_verbose() {
            let mut line = format!("{level}[{}]: {}", self.code, self.summary);
            if let Some(note) = &self.note {
                line.push_str(&format!(" (note: {note})"));
            } else if let Some(action) = &self.action {
                line.push_str(&format!(" (action: {action})"));
            }
            return writeln!(f, "{line}");
        }

        writeln!(f, "{level}[{}]: {}", self.code, self.summary)?;

        if let Some(command) = &self.command {
            writeln!(f, "  command: {command}")?;
        }

        for (key, value) in &self.context {
            writeln!(f, "  {key}: {value}")?;
        }

        if let Some(note) = &self.note {
            writeln!(f, "  note: {note}")?;
        }

        if let Some(action) = &self.action {
            write!(f, "  action: {action}")?;
        }

        Ok(())
    }
}

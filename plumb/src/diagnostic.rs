use std::fmt::{self, Display, Formatter};

use crate::verbosity::is_verbose;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warn,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    level: DiagnosticLevel,
    code: &'static str,
    summary: String,
    command: Option<String>,
    context: Vec<(String, String)>,
    hint: Option<String>,
    cause: Option<String>,
    action: Option<String>,
}

impl Diagnostic {
    pub fn error(code: &'static str, summary: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            code,
            summary: summary.into(),
            command: None,
            context: Vec::new(),
            hint: None,
            cause: None,
            action: None,
        }
    }

    pub fn warning(code: &'static str, summary: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warn,
            code,
            summary: summary.into(),
            command: None,
            context: Vec::new(),
            hint: None,
            cause: None,
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

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let level = match self.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warn => "warn",
        };
        if !is_verbose() {
            let mut line = format!("{level}[{}]: {}", self.code, self.summary);
            if let Some(hint) = &self.hint {
                line.push_str(&format!(" (hint: {hint})"));
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

        if let Some(hint) = &self.hint {
            writeln!(f, "  hint: {hint}")?;
        }

        if let Some(cause) = &self.cause {
            writeln!(f, "  cause: {cause}")?;
        }

        if let Some(action) = &self.action {
            write!(f, "  action: {action}")?;
        }

        Ok(())
    }
}

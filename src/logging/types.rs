use ratatui::style::{Color, Modifier, Style};

pub struct test {
    test: String,
}

impl test {
    pub fn new() {
        todo![];
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
    Debug,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
    pub timestamp: Option<std::time::SystemTime>,
    pub source: Option<String>,
    pub line_number: Option<u32>,
}

impl LogEntry {
    pub fn new(message: impl Into<String>, level: LogLevel) -> Self {
        Self {
            message: message.into(),
            level,
            timestamp: None,
            source: None,
            line_number: None,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, LogLevel::Info)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, LogLevel::Error)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, LogLevel::Warning)
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, LogLevel::Success)
    }

    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(message, LogLevel::Debug)
    }

    pub fn formatted_text(&self) -> String {
        let prefix = match self.level {
            LogLevel::Info => "[INFO]",
            LogLevel::Warning => "[WARNING]",
            LogLevel::Error => "[ERROR]",
            LogLevel::Success => "[SUCCESS]",
            LogLevel::Debug => "[DEBUG]",
        };

        if let Some(source) = &self.source {
            format!("{} [{}] - {}", prefix, source, self.message)
        } else {
            format!("{} - {}", prefix, self.message)
        }
    }

    pub fn style(&self) -> Style {
        match self.level {
            LogLevel::Info => Style::default().fg(Color::Cyan),
            LogLevel::Warning => Style::default().fg(Color::Yellow),
            LogLevel::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            LogLevel::Success => Style::default().fg(Color::Green),
            LogLevel::Debug => Style::default().fg(Color::Gray),
        }
    }
}

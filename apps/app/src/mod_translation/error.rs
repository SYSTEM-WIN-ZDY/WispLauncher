use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// 抛给前端的稳定错误码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TranslateErrorCode {
    UnsafeArchivePath,
    SignedModRefused,
    InvalidArchive,
    MissingApiKey,
    ModelNotFound,
    EmptyModelResponse,
    InvalidModelResponse,
    PlaceholderMismatch,
    WritebackVerificationFailed,
    QualityHardErrors,
    WorkGraphNoExit,
    UnsupportedResource,
    SessionHandoff,
    Cancelled,
    Io,
    Config,
    AiDisabled,
    AiProviderDisabled,
    AiModelNotSelected,
    AiRequestFailed,
}

impl TranslateErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsafeArchivePath => "UNSAFE_ARCHIVE_PATH",
            Self::SignedModRefused => "SIGNED_MOD_REFUSED",
            Self::InvalidArchive => "INVALID_ARCHIVE",
            Self::MissingApiKey => "MISSING_API_KEY",
            Self::ModelNotFound => "MODEL_NOT_FOUND",
            Self::EmptyModelResponse => "EMPTY_MODEL_RESPONSE",
            Self::InvalidModelResponse => "INVALID_MODEL_RESPONSE",
            Self::PlaceholderMismatch => "PLACEHOLDER_MISMATCH",
            Self::WritebackVerificationFailed => {
                "WRITEBACK_VERIFICATION_FAILED"
            }
            Self::QualityHardErrors => "QUALITY_HARD_ERRORS",
            Self::WorkGraphNoExit => "WORK_GRAPH_NO_EXIT",
            Self::UnsupportedResource => "UNSUPPORTED_RESOURCE",
            Self::SessionHandoff => "SESSION_HANDOFF",
            Self::Cancelled => "CANCELLED",
            Self::Io => "IO_ERROR",
            Self::Config => "CONFIG_ERROR",
            Self::AiDisabled => "AI_DISABLED",
            Self::AiProviderDisabled => "AI_PROVIDER_DISABLED",
            Self::AiModelNotSelected => "AI_MODEL_NOT_SELECTED",
            Self::AiRequestFailed => "AI_REQUEST_FAILED",
        }
    }
}

impl fmt::Display for TranslateErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct TranslateError {
    pub code: TranslateErrorCode,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl TranslateError {
    pub fn new(code: TranslateErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(
        code: TranslateErrorCode,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::with_source(TranslateErrorCode::Io, context, source)
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new(TranslateErrorCode::Config, message)
    }

    /// 过 Tauri 边界时用的消息：code + 可读文本。
    pub fn user_message(&self) -> String {
        format!("{}: {}", self.code.as_str(), self.detail_message())
    }

    pub fn detail_message(&self) -> String {
        match self.source.as_ref() {
            Some(source) => format!("{}: {source}", self.message),
            None => self.message.clone(),
        }
    }
}

impl From<std::io::Error> for TranslateError {
    fn from(error: std::io::Error) -> Self {
        Self::io("mod translation I/O error", error)
    }
}

impl From<theseus::Error> for TranslateError {
    fn from(error: theseus::Error) -> Self {
        use theseus::ErrorKind;

        let message = error.to_string();
        let code = match error.raw.as_ref() {
            ErrorKind::InputError(text)
                if text.contains("AI features are disabled") =>
            {
                TranslateErrorCode::AiDisabled
            }
            ErrorKind::InputError(text) if text.contains("is disabled") => {
                TranslateErrorCode::AiProviderDisabled
            }
            ErrorKind::InputError(text)
                if text.contains("Select an AI model") =>
            {
                TranslateErrorCode::AiModelNotSelected
            }
            ErrorKind::InputError(_) => TranslateErrorCode::Config,
            ErrorKind::StdIOError(_)
            | ErrorKind::IOError(_)
            | ErrorKind::FSError(_)
            | ErrorKind::Sqlx(_) => TranslateErrorCode::Io,
            _ => TranslateErrorCode::AiRequestFailed,
        };
        Self {
            code,
            message,
            source: None,
        }
    }
}

pub type Result<T> = std::result::Result<T, TranslateError>;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::remote::{RemoteError, RemoteErrorKind};

/// Errors returned to the UI. Messages are Turkish because they are shown to the user as-is.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bu işlem için yönetici yetkisi gerekiyor.")]
    AccessDenied,
    #[error("Süreç bulunamadı; zaten kapanmış olabilir.")]
    NotFound,
    #[error("Korumalı sistem süreci kapatılamaz: {0}")]
    Protected(String),
    #[error("Süreç listesi değişmiş; yeniledikten sonra tekrar deneyin.")]
    Changed,
    /// Feedback send limit reached; minutes until the next send is allowed.
    #[error("Gönderim sınırına ulaşıldı; {0} dk sonra tekrar deneyin.")]
    RateLimited(u64),
    /// SSH remote view failure; `code` is a `RemoteErrorKind` for the UI, `detail` the ssh stderr.
    #[error("{detail}")]
    Remote {
        code: RemoteErrorKind,
        detail: String,
    },
    #[error("{0}")]
    Other(String),
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            AppError::AccessDenied => "accessDenied",
            AppError::NotFound => "notFound",
            AppError::Protected(_) => "protected",
            AppError::Changed => "changed",
            AppError::RateLimited(_) => "rateLimited",
            AppError::Remote { .. } => "remote",
            AppError::Other(_) => "other",
        }
    }
}

/// peron-cli's kill results map onto the local kinds, so the UI treats both the same.
impl From<RemoteError> for AppError {
    fn from(e: RemoteError) -> Self {
        match e.kind {
            RemoteErrorKind::AccessDenied => AppError::AccessDenied,
            RemoteErrorKind::NotFound => AppError::NotFound,
            RemoteErrorKind::Changed => AppError::Changed,
            RemoteErrorKind::Protected => AppError::Protected(e.detail),
            code => AppError::Remote {
                code,
                detail: e.detail,
            },
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // The UI translates by `kind` (+ `detail`); `message` is the Turkish fallback.
        let detail = match self {
            AppError::Protected(name) => Some(name.clone()),
            AppError::Other(msg) => Some(msg.clone()),
            AppError::RateLimited(minutes) => Some(minutes.to_string()),
            AppError::Remote { detail, .. } => Some(detail.clone()),
            _ => None,
        };
        let code = match self {
            AppError::Remote { code, .. } => Some(*code),
            _ => None,
        };
        let mut s = serializer.serialize_struct("AppError", 4)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("detail", &detail)?;
        s.serialize_field("code", &code)?;
        s.end()
    }
}

#[cfg(windows)]
impl From<windows::core::Error> for AppError {
    fn from(e: windows::core::Error) -> Self {
        use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER};
        let code = e.code();
        if code == ERROR_ACCESS_DENIED.to_hresult() {
            AppError::AccessDenied
        } else if code == ERROR_INVALID_PARAMETER.to_hresult() {
            AppError::NotFound
        } else {
            AppError::Other(e.message().to_string())
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

use super::*;

use acme_kit::text;

/// Where a fallback release exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReleaseExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackReleaseExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FallbackReleaseExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback release exception as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseException {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub ttl_seconds: i64,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub issuer: String,
    pub grace_days: i64,
    pub maximum: f64,
    pub status: FallbackReleaseExceptionStatus,
}

impl FallbackReleaseException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReleaseExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReleaseExceptionStatus) -> bool {
        match self.status {
            FallbackReleaseExceptionStatus::New => {
                match next {
                    FallbackReleaseExceptionStatus::Queued | FallbackReleaseExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReleaseExceptionStatus::Queued => {
                return next == FallbackReleaseExceptionStatus::Done;
            }
            FallbackReleaseExceptionStatus::Running => {
                match next {
                    FallbackReleaseExceptionStatus::Done | FallbackReleaseExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReleaseExceptionStatus::Done => {
                match next {
                    FallbackReleaseExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackReleaseExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback release exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReleaseException {
        FallbackReleaseException {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            ttl_seconds: self.ttl_seconds,
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback release exception in status new.
pub fn new_fallback_release_exception(code: &str, name: &str) -> FallbackReleaseException {
    FallbackReleaseException {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        ttl_seconds: 0,
        batch_size: 0,
        tags: Vec::new(),
        issuer: "".to_string(),
        grace_days: 0,
        maximum: 0.0,
        status: FallbackReleaseExceptionStatus::New,
    }
}

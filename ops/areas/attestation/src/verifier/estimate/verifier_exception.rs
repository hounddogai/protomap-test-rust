use super::*;

use acme_kit::text;

/// Where a verifier exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for VerifierExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierException {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub maximum: f64,
    pub expires_at: i64,
    pub archived: bool,
    pub tags: Vec<String>,
    pub threshold: f64,
    pub budget: f64,
    pub batch_size: i64,
    pub ratio: f64,
    pub status: VerifierExceptionStatus,
}

impl VerifierException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierExceptionStatus) -> bool {
        match self.status {
            VerifierExceptionStatus::Planned => {
                match next {
                    VerifierExceptionStatus::InProgress | VerifierExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierExceptionStatus::Scheduled => {
                match next {
                    VerifierExceptionStatus::InProgress | VerifierExceptionStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierExceptionStatus::InProgress => {
                match next {
                    VerifierExceptionStatus::Completed | VerifierExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierExceptionStatus::Completed => {
                match next {
                    VerifierExceptionStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: VerifierExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier exception", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ratio + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierException {
        VerifierException {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            maximum: self.maximum,
            expires_at: self.expires_at,
            archived: self.archived,
            tags: self.tags.clone(),
            threshold: self.threshold,
            budget: self.budget,
            batch_size: self.batch_size,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a verifier exception in status planned.
pub fn new_verifier_exception(code: &str, title: &str) -> VerifierException {
    VerifierException {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        maximum: 0.0,
        expires_at: 0,
        archived: false,
        tags: Vec::new(),
        threshold: 0.0,
        budget: 0.0,
        batch_size: 0,
        ratio: 0.0,
        status: VerifierExceptionStatus::Planned,
    }
}

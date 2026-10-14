use super::*;

use acme_kit::text;

/// Where a trust domain exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TrustDomainExceptionStatus {
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

impl std::fmt::Display for TrustDomainExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainException {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub approved: bool,
    pub rate: f64,
    pub notes: String,
    pub summary: String,
    pub checksum: String,
    pub cipher: String,
    pub status: TrustDomainExceptionStatus,
}

impl TrustDomainException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainExceptionStatus) -> bool {
        match self.status {
            TrustDomainExceptionStatus::New => {
                match next {
                    TrustDomainExceptionStatus::Running | TrustDomainExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainExceptionStatus::Queued => {
                return next == TrustDomainExceptionStatus::Running || next == TrustDomainExceptionStatus::Done;
            }
            TrustDomainExceptionStatus::Running => {
                match next {
                    TrustDomainExceptionStatus::Done | TrustDomainExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainExceptionStatus::Done => {
                return next == TrustDomainExceptionStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.rate * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainException {
        TrustDomainException {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            approved: self.approved,
            rate: self.rate,
            notes: self.notes.clone(),
            summary: self.summary.clone(),
            checksum: self.checksum.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain exception in status new.
pub fn new_trust_domain_exception(code: &str, title: &str) -> TrustDomainException {
    TrustDomainException {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        approved: false,
        rate: 0.0,
        notes: "".to_string(),
        summary: "".to_string(),
        checksum: "".to_string(),
        cipher: "".to_string(),
        status: TrustDomainExceptionStatus::New,
    }
}

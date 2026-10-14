use super::*;

use acme_kit::text;

/// Where a failover draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FailoverDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FailoverDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDraft {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub version: i64,
    pub audience: String,
    pub cluster: String,
    pub status: FailoverDraftStatus,
}

impl FailoverDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverDraftStatus) -> bool {
        match self.status {
            FailoverDraftStatus::Open => {
                return next == FailoverDraftStatus::Reopened || next == FailoverDraftStatus::Archived;
            }
            FailoverDraftStatus::InReview => {
                return next == FailoverDraftStatus::Resolved;
            }
            FailoverDraftStatus::Resolved => {
                match next {
                    FailoverDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverDraftStatus::Reopened => {
                return next == FailoverDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> FailoverDraft {
        FailoverDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            version: self.version,
            audience: self.audience.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a failover draft in status open.
pub fn new_failover_draft(code: &str, title: &str) -> FailoverDraft {
    FailoverDraft {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        version: 0,
        audience: "".to_string(),
        cluster: "".to_string(),
        status: FailoverDraftStatus::Open,
    }
}

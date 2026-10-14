use super::*;

use acme_kit::text;

/// Where a isolation entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl IsolationEntryStatus {
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

impl std::fmt::Display for IsolationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation entry as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntry {
    pub code: String,
    pub label: String,
    pub source: String,
    pub principal: String,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub status: IsolationEntryStatus,
}

impl IsolationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationEntryStatus) -> bool {
        match self.status {
            IsolationEntryStatus::Open => {
                return next == IsolationEntryStatus::Reopened;
            }
            IsolationEntryStatus::InReview => {
                match next {
                    IsolationEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationEntryStatus::Resolved => {
                match next {
                    IsolationEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationEntryStatus::Reopened => {
                return next == IsolationEntryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationEntry {
        IsolationEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a isolation entry in status open.
pub fn new_isolation_entry(code: &str, label: &str) -> IsolationEntry {
    IsolationEntry {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        principal: "".to_string(),
        ttl_seconds: 0,
        retries: 0,
        status: IsolationEntryStatus::Open,
    }
}

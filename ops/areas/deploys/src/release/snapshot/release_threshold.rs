use super::*;

use acme_kit::text;

/// Where a release threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseThresholdStatus {
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

impl std::fmt::Display for ReleaseThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release threshold as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThreshold {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub scope: String,
    pub checksum: String,
    pub status: ReleaseThresholdStatus,
}

impl ReleaseThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseThresholdStatus) -> bool {
        match self.status {
            ReleaseThresholdStatus::Open => {
                return next == ReleaseThresholdStatus::Resolved;
            }
            ReleaseThresholdStatus::InReview => {
                match next {
                    ReleaseThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseThresholdStatus::Resolved => {
                match next {
                    ReleaseThresholdStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseThresholdStatus::Reopened => {
                return next == ReleaseThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "release threshold", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> ReleaseThreshold {
        ReleaseThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a release threshold in status open.
pub fn new_release_threshold(code: &str, label: &str) -> ReleaseThreshold {
    ReleaseThreshold {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        scope: "".to_string(),
        checksum: "".to_string(),
        status: ReleaseThresholdStatus::Open,
    }
}

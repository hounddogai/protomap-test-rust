use super::*;

use acme_kit::text;

/// Where a legacy usage audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyUsageAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyUsageAuditStatus {
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

impl std::fmt::Display for LegacyUsageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy usage audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAudit {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub ratio: f64,
    pub threshold: f64,
    pub pinned: bool,
    pub ceiling: f64,
    pub sequence: i64,
    pub status: LegacyUsageAuditStatus,
}

impl LegacyUsageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyUsageAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyUsageAuditStatus) -> bool {
        match self.status {
            LegacyUsageAuditStatus::Open => {
                match next {
                    LegacyUsageAuditStatus::InReview | LegacyUsageAuditStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyUsageAuditStatus::InReview => {
                return next == LegacyUsageAuditStatus::Resolved || next == LegacyUsageAuditStatus::Archived;
            }
            LegacyUsageAuditStatus::Resolved => {
                return next == LegacyUsageAuditStatus::Reopened || next == LegacyUsageAuditStatus::Archived;
            }
            LegacyUsageAuditStatus::Reopened => {
                return next == LegacyUsageAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyUsageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy usage audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.threshold - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyUsageAudit {
        LegacyUsageAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            ratio: self.ratio,
            threshold: self.threshold,
            pinned: self.pinned,
            ceiling: self.ceiling,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy usage audit in status open.
pub fn new_legacy_usage_audit(code: &str, label: &str) -> LegacyUsageAudit {
    LegacyUsageAudit {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        ratio: 0.0,
        threshold: 0.0,
        pinned: false,
        ceiling: 0.0,
        sequence: 0,
        status: LegacyUsageAuditStatus::Open,
    }
}

use super::*;

use acme_kit::text;

/// Where a pending runbook variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRunbookVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingRunbookVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingRunbookVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending runbook variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVariance {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub limit: i64,
    pub margin: f64,
    pub channel: String,
    pub minimum: f64,
    pub status: PendingRunbookVarianceStatus,
}

impl PendingRunbookVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRunbookVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRunbookVarianceStatus) -> bool {
        match self.status {
            PendingRunbookVarianceStatus::Proposed => {
                match next {
                    PendingRunbookVarianceStatus::Approved | PendingRunbookVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRunbookVarianceStatus::Approved => {
                return next == PendingRunbookVarianceStatus::Rejected
                    || next == PendingRunbookVarianceStatus::Withdrawn;
            }
            PendingRunbookVarianceStatus::Rejected => {
                return next == PendingRunbookVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRunbookVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending runbook variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.margin + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRunbookVariance {
        PendingRunbookVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            limit: self.limit,
            margin: self.margin,
            channel: self.channel.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending runbook variance in status proposed.
pub fn new_pending_runbook_variance(code: &str, title: &str) -> PendingRunbookVariance {
    PendingRunbookVariance {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        limit: 0,
        margin: 0.0,
        channel: "".to_string(),
        minimum: 0.0,
        status: PendingRunbookVarianceStatus::Proposed,
    }
}

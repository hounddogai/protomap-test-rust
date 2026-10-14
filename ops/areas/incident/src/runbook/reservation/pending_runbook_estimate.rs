use super::*;

use acme_kit::text;

/// Where a pending runbook estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRunbookEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingRunbookEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingRunbookEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending runbook estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookEstimate {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub cluster: String,
    pub namespace: String,
    pub status: PendingRunbookEstimateStatus,
}

impl PendingRunbookEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRunbookEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRunbookEstimateStatus) -> bool {
        match self.status {
            PendingRunbookEstimateStatus::Proposed => {
                match next {
                    PendingRunbookEstimateStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRunbookEstimateStatus::Approved => {
                match next {
                    PendingRunbookEstimateStatus::Rejected | PendingRunbookEstimateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRunbookEstimateStatus::Rejected => {
                return next == PendingRunbookEstimateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRunbookEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending runbook estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> PendingRunbookEstimate {
        PendingRunbookEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            cluster: self.cluster.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending runbook estimate in status proposed.
pub fn new_pending_runbook_estimate(code: &str, name: &str) -> PendingRunbookEstimate {
    PendingRunbookEstimate {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        cluster: "".to_string(),
        namespace: "".to_string(),
        status: PendingRunbookEstimateStatus::Proposed,
    }
}

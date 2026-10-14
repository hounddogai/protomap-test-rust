use super::*;

use acme_kit::text;

/// Where a pending health check batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingHealthCheckBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingHealthCheckBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingHealthCheckBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending health check batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckBatch {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub segment: String,
    pub revision: i64,
    pub key_id: String,
    pub status: PendingHealthCheckBatchStatus,
}

impl PendingHealthCheckBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingHealthCheckBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingHealthCheckBatchStatus) -> bool {
        match self.status {
            PendingHealthCheckBatchStatus::Proposed => {
                return next == PendingHealthCheckBatchStatus::Withdrawn;
            }
            PendingHealthCheckBatchStatus::Approved => {
                match next {
                    PendingHealthCheckBatchStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckBatchStatus::Rejected => {
                match next {
                    PendingHealthCheckBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingHealthCheckBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending health check batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rate - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingHealthCheckBatch {
        PendingHealthCheckBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            segment: self.segment.clone(),
            revision: self.revision,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a pending health check batch in status proposed.
pub fn new_pending_health_check_batch(code: &str, title: &str) -> PendingHealthCheckBatch {
    PendingHealthCheckBatch {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        segment: "".to_string(),
        revision: 0,
        key_id: "".to_string(),
        status: PendingHealthCheckBatchStatus::Proposed,
    }
}

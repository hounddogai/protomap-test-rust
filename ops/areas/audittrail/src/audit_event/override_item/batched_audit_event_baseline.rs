use super::*;

use acme_kit::text;

/// Where a batched audit event baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAuditEventBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedAuditEventBaselineStatus {
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

impl std::fmt::Display for BatchedAuditEventBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched audit event baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBaseline {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub principal: String,
    pub owner_id: String,
    pub minimum: f64,
    pub updated_at: i64,
    pub external_ref: String,
    pub retries: i64,
    pub status: BatchedAuditEventBaselineStatus,
}

impl BatchedAuditEventBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAuditEventBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAuditEventBaselineStatus) -> bool {
        match self.status {
            BatchedAuditEventBaselineStatus::Planned => {
                return next == BatchedAuditEventBaselineStatus::Completed;
            }
            BatchedAuditEventBaselineStatus::Scheduled => {
                return next == BatchedAuditEventBaselineStatus::Completed;
            }
            BatchedAuditEventBaselineStatus::InProgress => {
                return next == BatchedAuditEventBaselineStatus::Completed
                    || next == BatchedAuditEventBaselineStatus::Cancelled;
            }
            BatchedAuditEventBaselineStatus::Completed => {
                return next == BatchedAuditEventBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAuditEventBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched audit event baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.retries as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAuditEventBaseline {
        BatchedAuditEventBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            principal: self.principal.clone(),
            owner_id: self.owner_id.clone(),
            minimum: self.minimum,
            updated_at: self.updated_at,
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched audit event baseline in status planned.
pub fn new_batched_audit_event_baseline(code: &str, title: &str) -> BatchedAuditEventBaseline {
    BatchedAuditEventBaseline {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        principal: "".to_string(),
        owner_id: "".to_string(),
        minimum: 0.0,
        updated_at: 0,
        external_ref: "".to_string(),
        retries: 0,
        status: BatchedAuditEventBaselineStatus::Planned,
    }
}

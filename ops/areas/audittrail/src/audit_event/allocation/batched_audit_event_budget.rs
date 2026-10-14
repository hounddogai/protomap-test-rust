use super::*;

use acme_kit::text;

/// Where a batched audit event budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAuditEventBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedAuditEventBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedAuditEventBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched audit event budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBudget {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub threshold: f64,
    pub title: String,
    pub cluster: String,
    pub cost: f64,
    pub status: BatchedAuditEventBudgetStatus,
}

impl BatchedAuditEventBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAuditEventBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAuditEventBudgetStatus) -> bool {
        match self.status {
            BatchedAuditEventBudgetStatus::Idle => {
                match next {
                    BatchedAuditEventBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAuditEventBudgetStatus::Warming => {
                return next == BatchedAuditEventBudgetStatus::Ready || next == BatchedAuditEventBudgetStatus::Draining;
            }
            BatchedAuditEventBudgetStatus::Ready => {
                return next == BatchedAuditEventBudgetStatus::Draining;
            }
            BatchedAuditEventBudgetStatus::Draining => {
                match next {
                    BatchedAuditEventBudgetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedAuditEventBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched audit event budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.threshold - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAuditEventBudget {
        BatchedAuditEventBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            threshold: self.threshold,
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched audit event budget in status idle.
pub fn new_batched_audit_event_budget(code: &str, label: &str) -> BatchedAuditEventBudget {
    BatchedAuditEventBudget {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        threshold: 0.0,
        title: "".to_string(),
        cluster: "".to_string(),
        cost: 0.0,
        status: BatchedAuditEventBudgetStatus::Idle,
    }
}

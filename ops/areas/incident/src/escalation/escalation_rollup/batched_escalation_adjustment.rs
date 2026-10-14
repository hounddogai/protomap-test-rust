use super::*;

use acme_kit::text;

/// Where a batched escalation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedEscalationAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedEscalationAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedEscalationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched escalation adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationAdjustment {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub score: f64,
    pub limit: i64,
    pub approved: bool,
    pub unit: String,
    pub title: String,
    pub capacity: i64,
    pub burst: i64,
    pub status: BatchedEscalationAdjustmentStatus,
}

impl BatchedEscalationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedEscalationAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedEscalationAdjustmentStatus) -> bool {
        match self.status {
            BatchedEscalationAdjustmentStatus::Draft => {
                match next {
                    BatchedEscalationAdjustmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedEscalationAdjustmentStatus::Pending => {
                return next == BatchedEscalationAdjustmentStatus::Closed;
            }
            BatchedEscalationAdjustmentStatus::Active => {
                return next == BatchedEscalationAdjustmentStatus::Suspended
                    || next == BatchedEscalationAdjustmentStatus::Closed;
            }
            BatchedEscalationAdjustmentStatus::Suspended => {
                return next == BatchedEscalationAdjustmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedEscalationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched escalation adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.limit as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedEscalationAdjustment {
        BatchedEscalationAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            score: self.score,
            limit: self.limit,
            approved: self.approved,
            unit: self.unit.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a batched escalation adjustment in status draft.
pub fn new_batched_escalation_adjustment(code: &str, name: &str) -> BatchedEscalationAdjustment {
    BatchedEscalationAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        score: 0.0,
        limit: 0,
        approved: false,
        unit: "".to_string(),
        title: "".to_string(),
        capacity: 0,
        burst: 0,
        status: BatchedEscalationAdjustmentStatus::Draft,
    }
}

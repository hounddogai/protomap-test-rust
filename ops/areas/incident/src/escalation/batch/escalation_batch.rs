use super::*;

use acme_kit::text;

/// Where a escalation batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EscalationBatchStatus {
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

impl std::fmt::Display for EscalationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBatch {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub segment: String,
    pub weight: f64,
    pub timezone: String,
    pub approved: bool,
    pub grace_days: i64,
    pub visible: bool,
    pub status: EscalationBatchStatus,
}

impl EscalationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationBatchStatus) -> bool {
        match self.status {
            EscalationBatchStatus::Planned => {
                return next == EscalationBatchStatus::InProgress;
            }
            EscalationBatchStatus::Scheduled => {
                return next == EscalationBatchStatus::Cancelled;
            }
            EscalationBatchStatus::InProgress => {
                return next == EscalationBatchStatus::Cancelled;
            }
            EscalationBatchStatus::Completed => {
                return next == EscalationBatchStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.grace_days as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationBatch {
        EscalationBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            segment: self.segment.clone(),
            weight: self.weight,
            timezone: self.timezone.clone(),
            approved: self.approved,
            grace_days: self.grace_days,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a escalation batch in status planned.
pub fn new_escalation_batch(code: &str, name: &str) -> EscalationBatch {
    EscalationBatch {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        segment: "".to_string(),
        weight: 0.0,
        timezone: "".to_string(),
        approved: false,
        grace_days: 0,
        visible: false,
        status: EscalationBatchStatus::Planned,
    }
}

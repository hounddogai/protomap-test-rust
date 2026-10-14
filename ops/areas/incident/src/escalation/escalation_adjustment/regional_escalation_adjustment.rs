use super::*;

use acme_kit::text;

/// Where a regional escalation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalEscalationAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalEscalationAdjustmentStatus {
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

impl std::fmt::Display for RegionalEscalationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional escalation adjustment as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationAdjustment {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub budget: f64,
    pub segment: String,
    pub ttl_seconds: i64,
    pub currency: String,
    pub updated_at: i64,
    pub threshold: f64,
    pub status: RegionalEscalationAdjustmentStatus,
}

impl RegionalEscalationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalEscalationAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalEscalationAdjustmentStatus) -> bool {
        match self.status {
            RegionalEscalationAdjustmentStatus::Planned => {
                return next == RegionalEscalationAdjustmentStatus::Completed
                    || next == RegionalEscalationAdjustmentStatus::Cancelled;
            }
            RegionalEscalationAdjustmentStatus::Scheduled => {
                return next == RegionalEscalationAdjustmentStatus::Cancelled;
            }
            RegionalEscalationAdjustmentStatus::InProgress => {
                return next == RegionalEscalationAdjustmentStatus::Cancelled;
            }
            RegionalEscalationAdjustmentStatus::Completed => {
                match next {
                    RegionalEscalationAdjustmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalEscalationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional escalation adjustment {} cannot move from {} to {}",
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
        format!("{} {}", "regional escalation adjustment", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.updated_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalEscalationAdjustment {
        RegionalEscalationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            budget: self.budget,
            segment: self.segment.clone(),
            ttl_seconds: self.ttl_seconds,
            currency: self.currency.clone(),
            updated_at: self.updated_at,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional escalation adjustment in status planned.
pub fn new_regional_escalation_adjustment(code: &str, title: &str) -> RegionalEscalationAdjustment {
    RegionalEscalationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        budget: 0.0,
        segment: "".to_string(),
        ttl_seconds: 0,
        currency: "".to_string(),
        updated_at: 0,
        threshold: 0.0,
        status: RegionalEscalationAdjustmentStatus::Planned,
    }
}

use super::*;

use acme_kit::text;

/// Where a rolling verifier slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingVerifierSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingVerifierSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingVerifierSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling verifier slot as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSlot {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub score: f64,
    pub ceiling: f64,
    pub grace_days: i64,
    pub discount: f64,
    pub sequence: i64,
    pub unit: String,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub status: RollingVerifierSlotStatus,
}

impl RollingVerifierSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingVerifierSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingVerifierSlotStatus) -> bool {
        match self.status {
            RollingVerifierSlotStatus::Proposed => {
                return next == RollingVerifierSlotStatus::Approved || next == RollingVerifierSlotStatus::Withdrawn;
            }
            RollingVerifierSlotStatus::Approved => {
                return next == RollingVerifierSlotStatus::Withdrawn;
            }
            RollingVerifierSlotStatus::Rejected => {
                match next {
                    RollingVerifierSlotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingVerifierSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling verifier slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling verifier slot", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.attempts as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingVerifierSlot {
        RollingVerifierSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            score: self.score,
            ceiling: self.ceiling,
            grace_days: self.grace_days,
            discount: self.discount,
            sequence: self.sequence,
            unit: self.unit.clone(),
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling verifier slot in status proposed.
pub fn new_rolling_verifier_slot(code: &str, name: &str) -> RollingVerifierSlot {
    RollingVerifierSlot {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        score: 0.0,
        ceiling: 0.0,
        grace_days: 0,
        discount: 0.0,
        sequence: 0,
        unit: "".to_string(),
        ttl_seconds: 0,
        attempts: 0,
        status: RollingVerifierSlotStatus::Proposed,
    }
}

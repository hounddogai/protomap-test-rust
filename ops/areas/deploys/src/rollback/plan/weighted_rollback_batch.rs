use super::*;

use acme_kit::text;

/// Where a weighted rollback batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRollbackBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedRollbackBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedRollbackBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted rollback batch as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBatch {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub fingerprint: String,
    pub audience: String,
    pub reviewer: String,
    pub priority: i64,
    pub unit: String,
    pub status: WeightedRollbackBatchStatus,
}

impl WeightedRollbackBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRollbackBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRollbackBatchStatus) -> bool {
        match self.status {
            WeightedRollbackBatchStatus::Requested => {
                match next {
                    WeightedRollbackBatchStatus::Granted | WeightedRollbackBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRollbackBatchStatus::Granted => {
                return next == WeightedRollbackBatchStatus::Revoked || next == WeightedRollbackBatchStatus::Expired;
            }
            WeightedRollbackBatchStatus::Revoked => {
                match next {
                    WeightedRollbackBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: WeightedRollbackBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rollback batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted rollback batch", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.floor_value + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRollbackBatch {
        WeightedRollbackBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            reviewer: self.reviewer.clone(),
            priority: self.priority,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a weighted rollback batch in status requested.
pub fn new_weighted_rollback_batch(code: &str, name: &str) -> WeightedRollbackBatch {
    WeightedRollbackBatch {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        reviewer: "".to_string(),
        priority: 0,
        unit: "".to_string(),
        status: WeightedRollbackBatchStatus::Requested,
    }
}

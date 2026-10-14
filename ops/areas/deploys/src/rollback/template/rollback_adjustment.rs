use super::*;

use acme_kit::text;

/// Where a rollback adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollbackAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollbackAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustment {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub floor_value: f64,
    pub capacity: i64,
    pub quantity: i64,
    pub score: f64,
    pub principal: String,
    pub weight: f64,
    pub status: RollbackAdjustmentStatus,
}

impl RollbackAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackAdjustmentStatus) -> bool {
        match self.status {
            RollbackAdjustmentStatus::New => {
                return next == RollbackAdjustmentStatus::Queued;
            }
            RollbackAdjustmentStatus::Queued => {
                return next == RollbackAdjustmentStatus::Failed;
            }
            RollbackAdjustmentStatus::Running => {
                match next {
                    RollbackAdjustmentStatus::Done | RollbackAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackAdjustmentStatus::Done => {
                match next {
                    RollbackAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollbackAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback adjustment", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackAdjustment {
        RollbackAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            floor_value: self.floor_value,
            capacity: self.capacity,
            quantity: self.quantity,
            score: self.score,
            principal: self.principal.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback adjustment in status new.
pub fn new_rollback_adjustment(code: &str, label: &str) -> RollbackAdjustment {
    RollbackAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        floor_value: 0.0,
        capacity: 0,
        quantity: 0,
        score: 0.0,
        principal: "".to_string(),
        weight: 0.0,
        status: RollbackAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollback_adjustment("code-21", "label-948");
        assert_eq!(item.status, RollbackAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rollback_adjustment("code-21", "label-948");
        if item.move_to(RollbackAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollback_adjustment("code-21", "label-948");
        item.status = RollbackAdjustmentStatus::Failed;
        assert!(
            item.move_to(RollbackAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

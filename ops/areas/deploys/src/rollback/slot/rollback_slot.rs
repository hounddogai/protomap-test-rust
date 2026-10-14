use super::*;

use acme_kit::text;

/// Where a rollback slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollbackSlotStatus {
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

impl std::fmt::Display for RollbackSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback slot as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlot {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub description: String,
    pub unit: String,
    pub timezone: String,
    pub weight: f64,
    pub status: RollbackSlotStatus,
}

impl RollbackSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackSlotStatus) -> bool {
        match self.status {
            RollbackSlotStatus::New => {
                match next {
                    RollbackSlotStatus::Queued | RollbackSlotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSlotStatus::Queued => {
                match next {
                    RollbackSlotStatus::Running | RollbackSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSlotStatus::Running => {
                return next == RollbackSlotStatus::Done || next == RollbackSlotStatus::Failed;
            }
            RollbackSlotStatus::Done => {
                match next {
                    RollbackSlotStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollbackSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> RollbackSlot {
        RollbackSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            description: self.description.clone(),
            unit: self.unit.clone(),
            timezone: self.timezone.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rollback slot in status new.
pub fn new_rollback_slot(code: &str, label: &str) -> RollbackSlot {
    RollbackSlot {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        description: "".to_string(),
        unit: "".to_string(),
        timezone: "".to_string(),
        weight: 0.0,
        status: RollbackSlotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollback_slot("code-513", "label-100");
        assert_eq!(item.status, RollbackSlotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rollback_slot("code-513", "label-100");
        if item.move_to(RollbackSlotStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackSlotStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollback_slot("code-513", "label-100");
        item.status = RollbackSlotStatus::Failed;
        assert!(item.move_to(RollbackSlotStatus::New).is_err(), "expected move to fail");
    }
}

use super::*;

use acme_kit::text;

/// Where a rollback schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollbackScheduleStatus {
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

impl std::fmt::Display for RollbackScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSchedule {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub minimum: f64,
    pub region: String,
    pub status: RollbackScheduleStatus,
}

impl RollbackSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackScheduleStatus) -> bool {
        match self.status {
            RollbackScheduleStatus::Draft => {
                match next {
                    RollbackScheduleStatus::Active | RollbackScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackScheduleStatus::Pending => {
                match next {
                    RollbackScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackScheduleStatus::Active => {
                return next == RollbackScheduleStatus::Suspended;
            }
            RollbackScheduleStatus::Suspended => {
                match next {
                    RollbackScheduleStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollbackScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> RollbackSchedule {
        RollbackSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            minimum: self.minimum,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rollback schedule in status draft.
pub fn new_rollback_schedule(code: &str, label: &str) -> RollbackSchedule {
    RollbackSchedule {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        minimum: 0.0,
        region: "".to_string(),
        status: RollbackScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rollback_schedule("code-441", "label-844");
        assert_eq!(item.status, RollbackScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rollback_schedule("code-441", "label-844");
        if item.move_to(RollbackScheduleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackScheduleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rollback_schedule("code-441", "label-844");
        item.status = RollbackScheduleStatus::Closed;
        assert!(
            item.move_to(RollbackScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

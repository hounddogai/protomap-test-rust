use super::*;

use acme_kit::text;

/// Where a manual rollback rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRollbackRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualRollbackRollupStatus {
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

impl std::fmt::Display for ManualRollbackRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual rollback rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackRollup {
    pub code: String,
    pub name: String,
    pub country: String,
    pub floor_value: f64,
    pub limit: i64,
    pub capacity: i64,
    pub status: ManualRollbackRollupStatus,
}

impl ManualRollbackRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRollbackRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRollbackRollupStatus) -> bool {
        match self.status {
            ManualRollbackRollupStatus::Planned => {
                match next {
                    ManualRollbackRollupStatus::Scheduled | ManualRollbackRollupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRollbackRollupStatus::Scheduled => {
                return next == ManualRollbackRollupStatus::Completed || next == ManualRollbackRollupStatus::Cancelled;
            }
            ManualRollbackRollupStatus::InProgress => {
                return next == ManualRollbackRollupStatus::Completed;
            }
            ManualRollbackRollupStatus::Completed => {
                match next {
                    ManualRollbackRollupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ManualRollbackRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rollback rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.capacity as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRollbackRollup {
        ManualRollbackRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual rollback rollup in status planned.
pub fn new_manual_rollback_rollup(code: &str, name: &str) -> ManualRollbackRollup {
    ManualRollbackRollup {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        floor_value: 0.0,
        limit: 0,
        capacity: 0,
        status: ManualRollbackRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_rollback_rollup("code-426", "name-9");
        assert_eq!(item.status, ManualRollbackRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_manual_rollback_rollup("code-426", "name-9");
        if item.move_to(ManualRollbackRollupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRollbackRollupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_rollback_rollup("code-426", "name-9");
        item.status = ManualRollbackRollupStatus::Cancelled;
        assert!(
            item.move_to(ManualRollbackRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

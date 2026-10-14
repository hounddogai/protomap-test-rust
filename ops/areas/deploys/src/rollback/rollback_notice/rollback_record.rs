use super::*;

use acme_kit::text;

/// Where a rollback record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollbackRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollbackRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRecord {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub name: String,
    pub category: String,
    pub segment: String,
    pub attempts: i64,
    pub principal: String,
    pub status: RollbackRecordStatus,
}

impl RollbackRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackRecordStatus) -> bool {
        match self.status {
            RollbackRecordStatus::Proposed => {
                return next == RollbackRecordStatus::Withdrawn;
            }
            RollbackRecordStatus::Approved => {
                match next {
                    RollbackRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackRecordStatus::Rejected => {
                match next {
                    RollbackRecordStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollbackRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> RollbackRecord {
        RollbackRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            name: self.name.clone(),
            category: self.category.clone(),
            segment: self.segment.clone(),
            attempts: self.attempts,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rollback record in status proposed.
pub fn new_rollback_record(code: &str, title: &str) -> RollbackRecord {
    RollbackRecord {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        name: "".to_string(),
        category: "".to_string(),
        segment: "".to_string(),
        attempts: 0,
        principal: "".to_string(),
        status: RollbackRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rollback_record("code-976", "title-721");
        assert_eq!(item.status, RollbackRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_rollback_record("code-976", "title-721");
        if item.move_to(RollbackRecordStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackRecordStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rollback_record("code-976", "title-721");
        item.status = RollbackRecordStatus::Withdrawn;
        assert!(
            item.move_to(RollbackRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

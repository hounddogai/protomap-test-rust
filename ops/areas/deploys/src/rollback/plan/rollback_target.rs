use super::*;

use acme_kit::text;

/// Where a rollback target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollbackTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollbackTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTarget {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub name: String,
    pub notes: String,
    pub starts_at: i64,
    pub contact_email: String,
    pub approved: bool,
    pub status: RollbackTargetStatus,
}

impl RollbackTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackTargetStatus) -> bool {
        match self.status {
            RollbackTargetStatus::Created => {
                return next == RollbackTargetStatus::Validated;
            }
            RollbackTargetStatus::Validated => {
                match next {
                    RollbackTargetStatus::Published | RollbackTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackTargetStatus::Published => {
                return next == RollbackTargetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback target", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.priority as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackTarget {
        RollbackTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            name: self.name.clone(),
            notes: self.notes.clone(),
            starts_at: self.starts_at,
            contact_email: self.contact_email.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback target in status created.
pub fn new_rollback_target(code: &str, label: &str) -> RollbackTarget {
    RollbackTarget {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        name: "".to_string(),
        notes: "".to_string(),
        starts_at: 0,
        contact_email: "".to_string(),
        approved: false,
        status: RollbackTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollback_target("code-375", "label-706");
        assert_eq!(item.status, RollbackTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rollback_target("code-375", "label-706");
        if item.move_to(RollbackTargetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackTargetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollback_target("code-375", "label-706");
        item.status = RollbackTargetStatus::Retired;
        assert!(
            item.move_to(RollbackTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

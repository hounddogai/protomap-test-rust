use super::*;

use acme_kit::text;

/// Where a migration assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration assignment as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignment {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub segment: String,
    pub issuer: String,
    pub currency: String,
    pub weight: f64,
    pub approved: bool,
    pub grace_days: i64,
    pub status: MigrationAssignmentStatus,
}

impl MigrationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationAssignmentStatus) -> bool {
        match self.status {
            MigrationAssignmentStatus::Proposed => {
                return next == MigrationAssignmentStatus::Approved || next == MigrationAssignmentStatus::Rejected;
            }
            MigrationAssignmentStatus::Approved => {
                match next {
                    MigrationAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationAssignmentStatus::Rejected => {
                return next == MigrationAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.weight - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationAssignment {
        MigrationAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            issuer: self.issuer.clone(),
            currency: self.currency.clone(),
            weight: self.weight,
            approved: self.approved,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a migration assignment in status proposed.
pub fn new_migration_assignment(code: &str, title: &str) -> MigrationAssignment {
    MigrationAssignment {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        segment: "".to_string(),
        issuer: "".to_string(),
        currency: "".to_string(),
        weight: 0.0,
        approved: false,
        grace_days: 0,
        status: MigrationAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_migration_assignment("code-374", "title-752");
        assert_eq!(item.status, MigrationAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_migration_assignment("code-374", "title-752");
        if item.move_to(MigrationAssignmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationAssignmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_migration_assignment("code-374", "title-752");
        item.status = MigrationAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(MigrationAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

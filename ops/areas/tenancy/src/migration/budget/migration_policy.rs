use super::*;

use acme_kit::text;

/// Where a migration policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPolicy {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub attempts: i64,
    pub host: String,
    pub timezone: String,
    pub currency: String,
    pub approved: bool,
    pub status: MigrationPolicyStatus,
}

impl MigrationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationPolicyStatus) -> bool {
        match self.status {
            MigrationPolicyStatus::Proposed => {
                return next == MigrationPolicyStatus::Approved;
            }
            MigrationPolicyStatus::Approved => {
                return next == MigrationPolicyStatus::Rejected || next == MigrationPolicyStatus::Withdrawn;
            }
            MigrationPolicyStatus::Rejected => {
                match next {
                    MigrationPolicyStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MigrationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration policy {} cannot move from {} to {}",
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
        self.attempts as f64
    }

    pub fn copy(&self) -> MigrationPolicy {
        MigrationPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            attempts: self.attempts,
            host: self.host.clone(),
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a migration policy in status proposed.
pub fn new_migration_policy(code: &str, label: &str) -> MigrationPolicy {
    MigrationPolicy {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        attempts: 0,
        host: "".to_string(),
        timezone: "".to_string(),
        currency: "".to_string(),
        approved: false,
        status: MigrationPolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_migration_policy("code-195", "label-311");
        assert_eq!(item.status, MigrationPolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_migration_policy("code-195", "label-311");
        if item.move_to(MigrationPolicyStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationPolicyStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_migration_policy("code-195", "label-311");
        item.status = MigrationPolicyStatus::Withdrawn;
        assert!(
            item.move_to(MigrationPolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

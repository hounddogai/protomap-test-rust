use super::*;

use acme_kit::text;

/// Where a manual policy assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPolicyAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualPolicyAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualPolicyAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual policy assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyAssignment {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub source: String,
    pub notes: String,
    pub floor_value: f64,
    pub status: ManualPolicyAssignmentStatus,
}

impl ManualPolicyAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPolicyAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPolicyAssignmentStatus) -> bool {
        match self.status {
            ManualPolicyAssignmentStatus::Created => {
                match next {
                    ManualPolicyAssignmentStatus::Validated | ManualPolicyAssignmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPolicyAssignmentStatus::Validated => {
                return next == ManualPolicyAssignmentStatus::Published;
            }
            ManualPolicyAssignmentStatus::Published => {
                return next == ManualPolicyAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualPolicyAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual policy assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual policy assignment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> ManualPolicyAssignment {
        ManualPolicyAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a manual policy assignment in status created.
pub fn new_manual_policy_assignment(code: &str, name: &str) -> ManualPolicyAssignment {
    ManualPolicyAssignment {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        notes: "".to_string(),
        floor_value: 0.0,
        status: ManualPolicyAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_policy_assignment("code-79", "name-550");
        assert_eq!(item.status, ManualPolicyAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_manual_policy_assignment("code-79", "name-550");
        if item.move_to(ManualPolicyAssignmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPolicyAssignmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_policy_assignment("code-79", "name-550");
        item.status = ManualPolicyAssignmentStatus::Retired;
        assert!(
            item.move_to(ManualPolicyAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

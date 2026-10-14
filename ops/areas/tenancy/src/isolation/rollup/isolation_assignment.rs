use super::*;

use acme_kit::text;

/// Where a isolation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IsolationAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IsolationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationAssignment {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub grace_days: i64,
    pub weight: f64,
    pub batch_size: i64,
    pub status: IsolationAssignmentStatus,
}

impl IsolationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationAssignmentStatus) -> bool {
        match self.status {
            IsolationAssignmentStatus::Created => {
                return next == IsolationAssignmentStatus::Validated;
            }
            IsolationAssignmentStatus::Validated => {
                return next == IsolationAssignmentStatus::Published;
            }
            IsolationAssignmentStatus::Published => {
                match next {
                    IsolationAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: IsolationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "isolation assignment", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.weight + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationAssignment {
        IsolationAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            weight: self.weight,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a isolation assignment in status created.
pub fn new_isolation_assignment(code: &str, label: &str) -> IsolationAssignment {
    IsolationAssignment {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        grace_days: 0,
        weight: 0.0,
        batch_size: 0,
        status: IsolationAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_isolation_assignment("code-284", "label-725");
        assert_eq!(item.status, IsolationAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_isolation_assignment("code-284", "label-725");
        if item.move_to(IsolationAssignmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationAssignmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_isolation_assignment("code-284", "label-725");
        item.status = IsolationAssignmentStatus::Retired;
        assert!(
            item.move_to(IsolationAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

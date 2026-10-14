use super::*;

use acme_kit::text;

/// Where a suspension assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SuspensionAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SuspensionAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAssignment {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub unit: String,
    pub reviewer: String,
    pub summary: String,
    pub sequence: i64,
    pub key_id: String,
    pub timezone: String,
    pub issuer: String,
    pub status: SuspensionAssignmentStatus,
}

impl SuspensionAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionAssignmentStatus) -> bool {
        match self.status {
            SuspensionAssignmentStatus::Created => {
                return next == SuspensionAssignmentStatus::Validated || next == SuspensionAssignmentStatus::Published;
            }
            SuspensionAssignmentStatus::Validated => {
                match next {
                    SuspensionAssignmentStatus::Published | SuspensionAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAssignmentStatus::Published => {
                return next == SuspensionAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.capacity as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionAssignment {
        SuspensionAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            unit: self.unit.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            timezone: self.timezone.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a suspension assignment in status created.
pub fn new_suspension_assignment(code: &str, name: &str) -> SuspensionAssignment {
    SuspensionAssignment {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        unit: "".to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        sequence: 0,
        key_id: "".to_string(),
        timezone: "".to_string(),
        issuer: "".to_string(),
        status: SuspensionAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_suspension_assignment("code-43", "name-274");
        assert_eq!(item.status, SuspensionAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_suspension_assignment("code-43", "name-274");
        if item.move_to(SuspensionAssignmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionAssignmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_suspension_assignment("code-43", "name-274");
        item.status = SuspensionAssignmentStatus::Retired;
        assert!(
            item.move_to(SuspensionAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

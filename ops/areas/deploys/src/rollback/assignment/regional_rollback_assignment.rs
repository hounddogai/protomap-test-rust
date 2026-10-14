use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional rollback assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRollbackAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalRollbackAssignmentStatus {
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

impl std::fmt::Display for RegionalRollbackAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional rollback assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackAssignment {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub contact_email: String,
    pub discount: f64,
    pub unit: String,
    pub reviewer: String,
    pub attributes: BTreeMap<String, String>,
    pub batch_size: i64,
    pub rate: f64,
    pub weight: f64,
    pub status: RegionalRollbackAssignmentStatus,
}

impl RegionalRollbackAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRollbackAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRollbackAssignmentStatus) -> bool {
        match self.status {
            RegionalRollbackAssignmentStatus::New => {
                return next == RegionalRollbackAssignmentStatus::Done
                    || next == RegionalRollbackAssignmentStatus::Failed;
            }
            RegionalRollbackAssignmentStatus::Queued => {
                return next == RegionalRollbackAssignmentStatus::Running;
            }
            RegionalRollbackAssignmentStatus::Running => {
                return next == RegionalRollbackAssignmentStatus::Failed;
            }
            RegionalRollbackAssignmentStatus::Done => {
                return next == RegionalRollbackAssignmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRollbackAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional rollback assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.weight * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRollbackAssignment {
        RegionalRollbackAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            contact_email: self.contact_email.clone(),
            discount: self.discount,
            unit: self.unit.clone(),
            reviewer: self.reviewer.clone(),
            attributes: self.attributes.clone(),
            batch_size: self.batch_size,
            rate: self.rate,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a regional rollback assignment in status new.
pub fn new_regional_rollback_assignment(code: &str, label: &str) -> RegionalRollbackAssignment {
    RegionalRollbackAssignment {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        contact_email: "".to_string(),
        discount: 0.0,
        unit: "".to_string(),
        reviewer: "".to_string(),
        attributes: BTreeMap::new(),
        batch_size: 0,
        rate: 0.0,
        weight: 0.0,
        status: RegionalRollbackAssignmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_rollback_assignment("code-307", "label-513");
        assert_eq!(item.status, RegionalRollbackAssignmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_regional_rollback_assignment("code-307", "label-513");
        if item.move_to(RegionalRollbackAssignmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRollbackAssignmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_rollback_assignment("code-307", "label-513");
        item.status = RegionalRollbackAssignmentStatus::Failed;
        assert!(
            item.move_to(RegionalRollbackAssignmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a scheduled freeze assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFreezeAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledFreezeAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledFreezeAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled freeze assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeAssignment {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub notes: String,
    pub tags: Vec<String>,
    pub active: bool,
    pub currency: String,
    pub minimum: f64,
    pub reviewer: String,
    pub status: ScheduledFreezeAssignmentStatus,
}

impl ScheduledFreezeAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFreezeAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFreezeAssignmentStatus) -> bool {
        match self.status {
            ScheduledFreezeAssignmentStatus::Proposed => {
                match next {
                    ScheduledFreezeAssignmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFreezeAssignmentStatus::Approved => {
                match next {
                    ScheduledFreezeAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFreezeAssignmentStatus::Rejected => {
                return next == ScheduledFreezeAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFreezeAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled freeze assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> ScheduledFreezeAssignment {
        ScheduledFreezeAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            notes: self.notes.clone(),
            tags: self.tags.clone(),
            active: self.active,
            currency: self.currency.clone(),
            minimum: self.minimum,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled freeze assignment in status proposed.
pub fn new_scheduled_freeze_assignment(code: &str, title: &str) -> ScheduledFreezeAssignment {
    ScheduledFreezeAssignment {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        notes: "".to_string(),
        tags: Vec::new(),
        active: false,
        currency: "".to_string(),
        minimum: 0.0,
        reviewer: "".to_string(),
        status: ScheduledFreezeAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_freeze_assignment("code-123", "title-946");
        assert_eq!(item.status, ScheduledFreezeAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_scheduled_freeze_assignment("code-123", "title-946");
        if item.move_to(ScheduledFreezeAssignmentStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFreezeAssignmentStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_freeze_assignment("code-123", "title-946");
        item.status = ScheduledFreezeAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledFreezeAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

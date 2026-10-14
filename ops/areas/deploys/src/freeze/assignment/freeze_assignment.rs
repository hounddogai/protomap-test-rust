use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a freeze assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeAssignmentStatus {
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

impl std::fmt::Display for FreezeAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze assignment as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAssignment {
    pub code: String,
    pub name: String,
    pub label: String,
    pub summary: String,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub status: FreezeAssignmentStatus,
}

impl FreezeAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeAssignmentStatus) -> bool {
        match self.status {
            FreezeAssignmentStatus::New => {
                return next == FreezeAssignmentStatus::Running;
            }
            FreezeAssignmentStatus::Queued => {
                match next {
                    FreezeAssignmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeAssignmentStatus::Running => {
                match next {
                    FreezeAssignmentStatus::Done | FreezeAssignmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeAssignmentStatus::Done => {
                return next == FreezeAssignmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "freeze assignment", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> FreezeAssignment {
        FreezeAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze assignment in status new.
pub fn new_freeze_assignment(code: &str, name: &str) -> FreezeAssignment {
    FreezeAssignment {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        summary: "".to_string(),
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        amount: 0.0,
        status: FreezeAssignmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_freeze_assignment("code-163", "name-653");
        assert_eq!(item.status, FreezeAssignmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_freeze_assignment("code-163", "name-653");
        if item.move_to(FreezeAssignmentStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeAssignmentStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_freeze_assignment("code-163", "name-653");
        item.status = FreezeAssignmentStatus::Failed;
        assert!(
            item.move_to(FreezeAssignmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

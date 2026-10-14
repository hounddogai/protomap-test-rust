use super::*;

use acme_kit::text;

/// Where a binding checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BindingCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for BindingCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingCheckpoint {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub notes: String,
    pub summary: String,
    pub external_ref: String,
    pub timezone: String,
    pub description: String,
    pub label: String,
    pub status: BindingCheckpointStatus,
}

impl BindingCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingCheckpointStatus) -> bool {
        match self.status {
            BindingCheckpointStatus::Open => {
                match next {
                    BindingCheckpointStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingCheckpointStatus::InReview => {
                return next == BindingCheckpointStatus::Archived;
            }
            BindingCheckpointStatus::Resolved => {
                match next {
                    BindingCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingCheckpointStatus::Reopened => {
                match next {
                    BindingCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: BindingCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> BindingCheckpoint {
        BindingCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            notes: self.notes.clone(),
            summary: self.summary.clone(),
            external_ref: self.external_ref.clone(),
            timezone: self.timezone.clone(),
            description: self.description.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a binding checkpoint in status open.
pub fn new_binding_checkpoint(code: &str, name: &str) -> BindingCheckpoint {
    BindingCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        notes: "".to_string(),
        summary: "".to_string(),
        external_ref: "".to_string(),
        timezone: "".to_string(),
        description: "".to_string(),
        label: "".to_string(),
        status: BindingCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_binding_checkpoint("code-536", "name-146");
        assert_eq!(item.status, BindingCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_binding_checkpoint("code-536", "name-146");
        if item.move_to(BindingCheckpointStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingCheckpointStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_binding_checkpoint("code-536", "name-146");
        item.status = BindingCheckpointStatus::Archived;
        assert!(
            item.move_to(BindingCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

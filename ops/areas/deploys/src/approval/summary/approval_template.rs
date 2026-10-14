use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a approval template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ApprovalTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ApprovalTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval template as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTemplate {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub attributes: BTreeMap<String, String>,
    pub pinned: bool,
    pub capacity: i64,
    pub reviewer: String,
    pub status: ApprovalTemplateStatus,
}

impl ApprovalTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalTemplateStatus) -> bool {
        match self.status {
            ApprovalTemplateStatus::Planned => {
                match next {
                    ApprovalTemplateStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalTemplateStatus::Scheduled => {
                return next == ApprovalTemplateStatus::Cancelled;
            }
            ApprovalTemplateStatus::InProgress => {
                match next {
                    ApprovalTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalTemplateStatus::Completed => {
                return next == ApprovalTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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
        self.capacity as f64
    }

    pub fn copy(&self) -> ApprovalTemplate {
        ApprovalTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            attributes: self.attributes.clone(),
            pinned: self.pinned,
            capacity: self.capacity,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a approval template in status planned.
pub fn new_approval_template(code: &str, label: &str) -> ApprovalTemplate {
    ApprovalTemplate {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        attributes: BTreeMap::new(),
        pinned: false,
        capacity: 0,
        reviewer: "".to_string(),
        status: ApprovalTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_approval_template("code-878", "label-302");
        assert_eq!(item.status, ApprovalTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_approval_template("code-878", "label-302");
        if item.move_to(ApprovalTemplateStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalTemplateStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_approval_template("code-878", "label-302");
        item.status = ApprovalTemplateStatus::Cancelled;
        assert!(
            item.move_to(ApprovalTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

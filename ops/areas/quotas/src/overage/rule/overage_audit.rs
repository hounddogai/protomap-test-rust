use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a overage audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OverageAuditStatus {
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

impl std::fmt::Display for OverageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage audit as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageAudit {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub title: String,
    pub contact_email: String,
    pub attributes: BTreeMap<String, String>,
    pub status: OverageAuditStatus,
}

impl OverageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageAuditStatus) -> bool {
        match self.status {
            OverageAuditStatus::Open => {
                match next {
                    OverageAuditStatus::Resolved | OverageAuditStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageAuditStatus::InReview => {
                return next == OverageAuditStatus::Resolved || next == OverageAuditStatus::Reopened;
            }
            OverageAuditStatus::Resolved => {
                return next == OverageAuditStatus::Archived;
            }
            OverageAuditStatus::Reopened => {
                return next == OverageAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage audit {} cannot move from {} to {}",
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

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> OverageAudit {
        OverageAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a overage audit in status open.
pub fn new_overage_audit(code: &str, label: &str) -> OverageAudit {
    OverageAudit {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        title: "".to_string(),
        contact_email: "".to_string(),
        attributes: BTreeMap::new(),
        status: OverageAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_overage_audit("code-852", "label-910");
        assert_eq!(item.status, OverageAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_overage_audit("code-852", "label-910");
        if item.move_to(OverageAuditStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageAuditStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_overage_audit("code-852", "label-910");
        item.status = OverageAuditStatus::Archived;
        assert!(item.move_to(OverageAuditStatus::Open).is_err(), "expected move to fail");
    }
}

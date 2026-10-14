use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a approval draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ApprovalDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ApprovalDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraft {
    pub code: String,
    pub label: String,
    pub country: String,
    pub contact_email: String,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub key_id: String,
    pub starts_at: i64,
    pub approved: bool,
    pub amount: f64,
    pub status: ApprovalDraftStatus,
}

impl ApprovalDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalDraftStatus) -> bool {
        match self.status {
            ApprovalDraftStatus::Proposed => {
                return next == ApprovalDraftStatus::Withdrawn;
            }
            ApprovalDraftStatus::Approved => {
                return next == ApprovalDraftStatus::Rejected || next == ApprovalDraftStatus::Withdrawn;
            }
            ApprovalDraftStatus::Rejected => {
                match next {
                    ApprovalDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ApprovalDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn projected_load(&self) -> f64 {
        ((self.starts_at as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalDraft {
        ApprovalDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            contact_email: self.contact_email.clone(),
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            starts_at: self.starts_at,
            approved: self.approved,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a approval draft in status proposed.
pub fn new_approval_draft(code: &str, label: &str) -> ApprovalDraft {
    ApprovalDraft {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        contact_email: "".to_string(),
        attributes: BTreeMap::new(),
        category: "".to_string(),
        key_id: "".to_string(),
        starts_at: 0,
        approved: false,
        amount: 0.0,
        status: ApprovalDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_approval_draft("code-380", "label-362");
        assert_eq!(item.status, ApprovalDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_approval_draft("code-380", "label-362");
        if item.move_to(ApprovalDraftStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalDraftStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_approval_draft("code-380", "label-362");
        item.status = ApprovalDraftStatus::Withdrawn;
        assert!(
            item.move_to(ApprovalDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

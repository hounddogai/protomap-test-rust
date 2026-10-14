use super::*;

use acme_kit::text;

/// Where a isolation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl IsolationDraftStatus {
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

impl std::fmt::Display for IsolationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraft {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub description: String,
    pub budget: f64,
    pub tenant_id: String,
    pub capacity: i64,
    pub status: IsolationDraftStatus,
}

impl IsolationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationDraftStatus) -> bool {
        match self.status {
            IsolationDraftStatus::Open => {
                return next == IsolationDraftStatus::InReview;
            }
            IsolationDraftStatus::InReview => {
                match next {
                    IsolationDraftStatus::Resolved | IsolationDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationDraftStatus::Resolved => {
                match next {
                    IsolationDraftStatus::Reopened | IsolationDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationDraftStatus::Reopened => {
                return next == IsolationDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "isolation draft", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.capacity as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationDraft {
        IsolationDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            description: self.description.clone(),
            budget: self.budget,
            tenant_id: self.tenant_id.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a isolation draft in status open.
pub fn new_isolation_draft(code: &str, label: &str) -> IsolationDraft {
    IsolationDraft {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        description: "".to_string(),
        budget: 0.0,
        tenant_id: "".to_string(),
        capacity: 0,
        status: IsolationDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_isolation_draft("code-655", "label-972");
        assert_eq!(item.status, IsolationDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_isolation_draft("code-655", "label-972");
        if item.move_to(IsolationDraftStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationDraftStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_isolation_draft("code-655", "label-972");
        item.status = IsolationDraftStatus::Archived;
        assert!(
            item.move_to(IsolationDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict approval baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictApprovalBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictApprovalBaselineStatus {
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

impl std::fmt::Display for StrictApprovalBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict approval baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalBaseline {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub region: String,
    pub namespace: String,
    pub description: String,
    pub archived: bool,
    pub attributes: BTreeMap<String, String>,
    pub status: StrictApprovalBaselineStatus,
}

impl StrictApprovalBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictApprovalBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictApprovalBaselineStatus) -> bool {
        match self.status {
            StrictApprovalBaselineStatus::Open => {
                match next {
                    StrictApprovalBaselineStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalBaselineStatus::InReview => {
                return next == StrictApprovalBaselineStatus::Resolved;
            }
            StrictApprovalBaselineStatus::Resolved => {
                match next {
                    StrictApprovalBaselineStatus::Reopened | StrictApprovalBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalBaselineStatus::Reopened => {
                match next {
                    StrictApprovalBaselineStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictApprovalBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict approval baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict approval baseline", self.code)
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
        self.quota_units as f64
    }

    pub fn copy(&self) -> StrictApprovalBaseline {
        StrictApprovalBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            region: self.region.clone(),
            namespace: self.namespace.clone(),
            description: self.description.clone(),
            archived: self.archived,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a strict approval baseline in status open.
pub fn new_strict_approval_baseline(code: &str, label: &str) -> StrictApprovalBaseline {
    StrictApprovalBaseline {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        region: "".to_string(),
        namespace: "".to_string(),
        description: "".to_string(),
        archived: false,
        attributes: BTreeMap::new(),
        status: StrictApprovalBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_approval_baseline("code-383", "label-895");
        assert_eq!(item.status, StrictApprovalBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_strict_approval_baseline("code-383", "label-895");
        if item.move_to(StrictApprovalBaselineStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictApprovalBaselineStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_approval_baseline("code-383", "label-895");
        item.status = StrictApprovalBaselineStatus::Archived;
        assert!(
            item.move_to(StrictApprovalBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

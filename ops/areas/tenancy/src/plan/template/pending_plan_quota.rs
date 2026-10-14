use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending plan quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPlanQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingPlanQuotaStatus {
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

impl std::fmt::Display for PendingPlanQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending plan quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanQuota {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub namespace: String,
    pub reviewer: String,
    pub score: f64,
    pub approved: bool,
    pub status: PendingPlanQuotaStatus,
}

impl PendingPlanQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPlanQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPlanQuotaStatus) -> bool {
        match self.status {
            PendingPlanQuotaStatus::Open => {
                return next == PendingPlanQuotaStatus::Reopened || next == PendingPlanQuotaStatus::Archived;
            }
            PendingPlanQuotaStatus::InReview => {
                match next {
                    PendingPlanQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPlanQuotaStatus::Resolved => {
                return next == PendingPlanQuotaStatus::Archived;
            }
            PendingPlanQuotaStatus::Reopened => {
                return next == PendingPlanQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPlanQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending plan quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PendingPlanQuota {
        PendingPlanQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            namespace: self.namespace.clone(),
            reviewer: self.reviewer.clone(),
            score: self.score,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending plan quota in status open.
pub fn new_pending_plan_quota(code: &str, label: &str) -> PendingPlanQuota {
    PendingPlanQuota {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        namespace: "".to_string(),
        reviewer: "".to_string(),
        score: 0.0,
        approved: false,
        status: PendingPlanQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_plan_quota("code-806", "label-532");
        assert_eq!(item.status, PendingPlanQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_pending_plan_quota("code-806", "label-532");
        if item.move_to(PendingPlanQuotaStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPlanQuotaStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_plan_quota("code-806", "label-532");
        item.status = PendingPlanQuotaStatus::Archived;
        assert!(
            item.move_to(PendingPlanQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

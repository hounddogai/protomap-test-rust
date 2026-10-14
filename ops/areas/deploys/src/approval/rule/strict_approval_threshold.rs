use super::*;

use acme_kit::text;

/// Where a strict approval threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictApprovalThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictApprovalThresholdStatus {
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

impl std::fmt::Display for StrictApprovalThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict approval threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalThreshold {
    pub code: String,
    pub label: String,
    pub category: String,
    pub issuer: String,
    pub port: i64,
    pub title: String,
    pub tenant_id: String,
    pub principal: String,
    pub region: String,
    pub status: StrictApprovalThresholdStatus,
}

impl StrictApprovalThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictApprovalThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictApprovalThresholdStatus) -> bool {
        match self.status {
            StrictApprovalThresholdStatus::Open => {
                return next == StrictApprovalThresholdStatus::Resolved;
            }
            StrictApprovalThresholdStatus::InReview => {
                match next {
                    StrictApprovalThresholdStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalThresholdStatus::Resolved => {
                match next {
                    StrictApprovalThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalThresholdStatus::Reopened => {
                match next {
                    StrictApprovalThresholdStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictApprovalThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict approval threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict approval threshold", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> StrictApprovalThreshold {
        StrictApprovalThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            port: self.port,
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            principal: self.principal.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict approval threshold in status open.
pub fn new_strict_approval_threshold(code: &str, label: &str) -> StrictApprovalThreshold {
    StrictApprovalThreshold {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        issuer: "".to_string(),
        port: 0,
        title: "".to_string(),
        tenant_id: "".to_string(),
        principal: "".to_string(),
        region: "".to_string(),
        status: StrictApprovalThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_approval_threshold("code-997", "label-828");
        assert_eq!(item.status, StrictApprovalThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_strict_approval_threshold("code-997", "label-828");
        if item.move_to(StrictApprovalThresholdStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictApprovalThresholdStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_approval_threshold("code-997", "label-828");
        item.status = StrictApprovalThresholdStatus::Archived;
        assert!(
            item.move_to(StrictApprovalThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a approval estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ApprovalEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ApprovalEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One approval estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEstimate {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub expires_at: i64,
    pub amount: f64,
    pub scope: String,
    pub window_days: i64,
    pub status: ApprovalEstimateStatus,
}

impl ApprovalEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalEstimateStatus) -> bool {
        match self.status {
            ApprovalEstimateStatus::Created => {
                return next == ApprovalEstimateStatus::Published;
            }
            ApprovalEstimateStatus::Validated => {
                return next == ApprovalEstimateStatus::Published || next == ApprovalEstimateStatus::Retired;
            }
            ApprovalEstimateStatus::Published => {
                match next {
                    ApprovalEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: ApprovalEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.amount - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalEstimate {
        ApprovalEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            expires_at: self.expires_at,
            amount: self.amount,
            scope: self.scope.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a approval estimate in status created.
pub fn new_approval_estimate(code: &str, label: &str) -> ApprovalEstimate {
    ApprovalEstimate {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        expires_at: 0,
        amount: 0.0,
        scope: "".to_string(),
        window_days: 0,
        status: ApprovalEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_approval_estimate("code-135", "label-601");
        assert_eq!(item.status, ApprovalEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_approval_estimate("code-135", "label-601");
        if item.move_to(ApprovalEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_approval_estimate("code-135", "label-601");
        item.status = ApprovalEstimateStatus::Retired;
        assert!(
            item.move_to(ApprovalEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

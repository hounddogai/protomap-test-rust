use super::*;

use acme_kit::text;

/// Where a manual policy quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPolicyQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualPolicyQuotaStatus {
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

impl std::fmt::Display for ManualPolicyQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual policy quota as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyQuota {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub limit: i64,
    pub owner_id: String,
    pub status: ManualPolicyQuotaStatus,
}

impl ManualPolicyQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPolicyQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPolicyQuotaStatus) -> bool {
        match self.status {
            ManualPolicyQuotaStatus::Open => {
                match next {
                    ManualPolicyQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPolicyQuotaStatus::InReview => {
                match next {
                    ManualPolicyQuotaStatus::Reopened | ManualPolicyQuotaStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPolicyQuotaStatus::Resolved => {
                return next == ManualPolicyQuotaStatus::Reopened;
            }
            ManualPolicyQuotaStatus::Reopened => {
                match next {
                    ManualPolicyQuotaStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualPolicyQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual policy quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.limit as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPolicyQuota {
        ManualPolicyQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            limit: self.limit,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual policy quota in status open.
pub fn new_manual_policy_quota(code: &str, label: &str) -> ManualPolicyQuota {
    ManualPolicyQuota {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        limit: 0,
        owner_id: "".to_string(),
        status: ManualPolicyQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_policy_quota("code-427", "label-762");
        assert_eq!(item.status, ManualPolicyQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_manual_policy_quota("code-427", "label-762");
        if item.move_to(ManualPolicyQuotaStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPolicyQuotaStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_policy_quota("code-427", "label-762");
        item.status = ManualPolicyQuotaStatus::Archived;
        assert!(
            item.move_to(ManualPolicyQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

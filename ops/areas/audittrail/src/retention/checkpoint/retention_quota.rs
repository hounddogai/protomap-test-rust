use super::*;

use acme_kit::text;

/// Where a retention quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RetentionQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RetentionQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuota {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub scope: String,
    pub revision: i64,
    pub status: RetentionQuotaStatus,
}

impl RetentionQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionQuotaStatus) -> bool {
        match self.status {
            RetentionQuotaStatus::Proposed => {
                return next == RetentionQuotaStatus::Withdrawn;
            }
            RetentionQuotaStatus::Approved => {
                match next {
                    RetentionQuotaStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionQuotaStatus::Rejected => {
                return next == RetentionQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "retention quota", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.budget + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionQuota {
        RetentionQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            scope: self.scope.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a retention quota in status proposed.
pub fn new_retention_quota(code: &str, name: &str) -> RetentionQuota {
    RetentionQuota {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        scope: "".to_string(),
        revision: 0,
        status: RetentionQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_retention_quota("code-57", "name-551");
        assert_eq!(item.status, RetentionQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_retention_quota("code-57", "name-551");
        if item.move_to(RetentionQuotaStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionQuotaStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_retention_quota("code-57", "name-551");
        item.status = RetentionQuotaStatus::Withdrawn;
        assert!(
            item.move_to(RetentionQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a tiered approval quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredApprovalQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredApprovalQuotaStatus {
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

impl std::fmt::Display for TieredApprovalQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered approval quota as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalQuota {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub cost: f64,
    pub tenant_id: String,
    pub notes: String,
    pub quota_units: i64,
    pub revision: i64,
    pub attempts: i64,
    pub contact_email: String,
    pub floor_value: f64,
    pub status: TieredApprovalQuotaStatus,
}

impl TieredApprovalQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredApprovalQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredApprovalQuotaStatus) -> bool {
        match self.status {
            TieredApprovalQuotaStatus::Open => {
                match next {
                    TieredApprovalQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredApprovalQuotaStatus::InReview => {
                return next == TieredApprovalQuotaStatus::Resolved || next == TieredApprovalQuotaStatus::Archived;
            }
            TieredApprovalQuotaStatus::Resolved => {
                return next == TieredApprovalQuotaStatus::Reopened;
            }
            TieredApprovalQuotaStatus::Reopened => {
                return next == TieredApprovalQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredApprovalQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered approval quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered approval quota", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quota_units as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredApprovalQuota {
        TieredApprovalQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            notes: self.notes.clone(),
            quota_units: self.quota_units,
            revision: self.revision,
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered approval quota in status open.
pub fn new_tiered_approval_quota(code: &str, title: &str) -> TieredApprovalQuota {
    TieredApprovalQuota {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        cost: 0.0,
        tenant_id: "".to_string(),
        notes: "".to_string(),
        quota_units: 0,
        revision: 0,
        attempts: 0,
        contact_email: "".to_string(),
        floor_value: 0.0,
        status: TieredApprovalQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_approval_quota("code-814", "title-479");
        assert_eq!(item.status, TieredApprovalQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tiered_approval_quota("code-814", "title-479");
        if item.move_to(TieredApprovalQuotaStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredApprovalQuotaStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_approval_quota("code-814", "title-479");
        item.status = TieredApprovalQuotaStatus::Archived;
        assert!(
            item.move_to(TieredApprovalQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

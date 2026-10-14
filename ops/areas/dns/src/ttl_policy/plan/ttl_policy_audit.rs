use super::*;

use acme_kit::text;

/// Where a ttl policy audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TtlPolicyAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TtlPolicyAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy audit as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAudit {
    pub code: String,
    pub title: String,
    pub region: String,
    pub approved: bool,
    pub attempts: i64,
    pub tenant_id: String,
    pub grace_days: i64,
    pub status: TtlPolicyAuditStatus,
}

impl TtlPolicyAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyAuditStatus) -> bool {
        match self.status {
            TtlPolicyAuditStatus::Draft => {
                return next == TtlPolicyAuditStatus::Closed;
            }
            TtlPolicyAuditStatus::Pending => {
                return next == TtlPolicyAuditStatus::Active || next == TtlPolicyAuditStatus::Suspended;
            }
            TtlPolicyAuditStatus::Active => {
                match next {
                    TtlPolicyAuditStatus::Suspended | TtlPolicyAuditStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyAuditStatus::Suspended => {
                match next {
                    TtlPolicyAuditStatus::Closed => {
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

    pub fn move_to(&mut self, next: TtlPolicyAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.grace_days as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyAudit {
        TtlPolicyAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            approved: self.approved,
            attempts: self.attempts,
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a ttl policy audit in status draft.
pub fn new_ttl_policy_audit(code: &str, title: &str) -> TtlPolicyAudit {
    TtlPolicyAudit {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        approved: false,
        attempts: 0,
        tenant_id: "".to_string(),
        grace_days: 0,
        status: TtlPolicyAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_ttl_policy_audit("code-697", "title-853");
        assert_eq!(item.status, TtlPolicyAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_ttl_policy_audit("code-697", "title-853");
        if item.move_to(TtlPolicyAuditStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyAuditStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_ttl_policy_audit("code-697", "title-853");
        item.status = TtlPolicyAuditStatus::Closed;
        assert!(
            item.move_to(TtlPolicyAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

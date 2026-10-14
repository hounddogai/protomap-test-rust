use super::*;

use acme_kit::text;

/// Where a fallback tenant audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTenantAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackTenantAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackTenantAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback tenant audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAudit {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub approved: bool,
    pub scope: String,
    pub cluster: String,
    pub archived: bool,
    pub cipher: String,
    pub priority: i64,
    pub status: FallbackTenantAuditStatus,
}

impl FallbackTenantAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTenantAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTenantAuditStatus) -> bool {
        match self.status {
            FallbackTenantAuditStatus::Proposed => {
                match next {
                    FallbackTenantAuditStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTenantAuditStatus::Approved => {
                match next {
                    FallbackTenantAuditStatus::Rejected | FallbackTenantAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTenantAuditStatus::Rejected => {
                match next {
                    FallbackTenantAuditStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackTenantAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback tenant audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.priority as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackTenantAudit {
        FallbackTenantAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            approved: self.approved,
            scope: self.scope.clone(),
            cluster: self.cluster.clone(),
            archived: self.archived,
            cipher: self.cipher.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback tenant audit in status proposed.
pub fn new_fallback_tenant_audit(code: &str, title: &str) -> FallbackTenantAudit {
    FallbackTenantAudit {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        approved: false,
        scope: "".to_string(),
        cluster: "".to_string(),
        archived: false,
        cipher: "".to_string(),
        priority: 0,
        status: FallbackTenantAuditStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_tenant_audit("code-953", "title-328");
        assert_eq!(item.status, FallbackTenantAuditStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_fallback_tenant_audit("code-953", "title-328");
        if item.move_to(FallbackTenantAuditStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTenantAuditStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_tenant_audit("code-953", "title-328");
        item.status = FallbackTenantAuditStatus::Withdrawn;
        assert!(
            item.move_to(FallbackTenantAuditStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

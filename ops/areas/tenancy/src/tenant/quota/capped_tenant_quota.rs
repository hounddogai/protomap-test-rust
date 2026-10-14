use super::*;

use acme_kit::text;

/// Where a capped tenant quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTenantQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedTenantQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CappedTenantQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped tenant quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuota {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub rate: f64,
    pub ceiling: f64,
    pub scope: String,
    pub notes: String,
    pub attempts: i64,
    pub batch_size: i64,
    pub issuer: String,
    pub title: String,
    pub status: CappedTenantQuotaStatus,
}

impl CappedTenantQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTenantQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTenantQuotaStatus) -> bool {
        match self.status {
            CappedTenantQuotaStatus::New => {
                return next == CappedTenantQuotaStatus::Running;
            }
            CappedTenantQuotaStatus::Queued => {
                return next == CappedTenantQuotaStatus::Running;
            }
            CappedTenantQuotaStatus::Running => {
                return next == CappedTenantQuotaStatus::Failed;
            }
            CappedTenantQuotaStatus::Done => {
                match next {
                    CappedTenantQuotaStatus::Failed => {
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

    pub fn move_to(&mut self, next: CappedTenantQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped tenant quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped tenant quota", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rate - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTenantQuota {
        CappedTenantQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            rate: self.rate,
            ceiling: self.ceiling,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            attempts: self.attempts,
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a capped tenant quota in status new.
pub fn new_capped_tenant_quota(code: &str, name: &str) -> CappedTenantQuota {
    CappedTenantQuota {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        rate: 0.0,
        ceiling: 0.0,
        scope: "".to_string(),
        notes: "".to_string(),
        attempts: 0,
        batch_size: 0,
        issuer: "".to_string(),
        title: "".to_string(),
        status: CappedTenantQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_tenant_quota("code-153", "name-780");
        assert_eq!(item.status, CappedTenantQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_capped_tenant_quota("code-153", "name-780");
        if item.move_to(CappedTenantQuotaStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTenantQuotaStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_tenant_quota("code-153", "name-780");
        item.status = CappedTenantQuotaStatus::Failed;
        assert!(
            item.move_to(CappedTenantQuotaStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a federation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationAuditStatus {
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

impl std::fmt::Display for FederationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAudit {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub discount: f64,
    pub name: String,
    pub score: f64,
    pub tenant_id: String,
    pub approved: bool,
    pub capacity: i64,
    pub status: FederationAuditStatus,
}

impl FederationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationAuditStatus) -> bool {
        match self.status {
            FederationAuditStatus::New => {
                return next == FederationAuditStatus::Queued;
            }
            FederationAuditStatus::Queued => {
                return next == FederationAuditStatus::Running || next == FederationAuditStatus::Done;
            }
            FederationAuditStatus::Running => {
                return next == FederationAuditStatus::Done;
            }
            FederationAuditStatus::Done => {
                match next {
                    FederationAuditStatus::Failed => {
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

    pub fn move_to(&mut self, next: FederationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        (self.score * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationAudit {
        FederationAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            discount: self.discount,
            name: self.name.clone(),
            score: self.score,
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation audit in status new.
pub fn new_federation_audit(code: &str, title: &str) -> FederationAudit {
    FederationAudit {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        ttl_seconds: 0,
        discount: 0.0,
        name: "".to_string(),
        score: 0.0,
        tenant_id: "".to_string(),
        approved: false,
        capacity: 0,
        status: FederationAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_federation_audit("code-841", "title-401");
        assert_eq!(item.status, FederationAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_federation_audit("code-841", "title-401");
        if item.move_to(FederationAuditStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationAuditStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_federation_audit("code-841", "title-401");
        item.status = FederationAuditStatus::Failed;
        assert!(
            item.move_to(FederationAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

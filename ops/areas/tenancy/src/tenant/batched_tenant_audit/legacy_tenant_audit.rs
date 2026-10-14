use super::*;

use acme_kit::text;

/// Where a legacy tenant audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTenantAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyTenantAuditStatus {
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

impl std::fmt::Display for LegacyTenantAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy tenant audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAudit {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub external_ref: String,
    pub segment: String,
    pub revision: i64,
    pub unit: String,
    pub locale: String,
    pub status: LegacyTenantAuditStatus,
}

impl LegacyTenantAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTenantAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTenantAuditStatus) -> bool {
        match self.status {
            LegacyTenantAuditStatus::New => {
                match next {
                    LegacyTenantAuditStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTenantAuditStatus::Queued => {
                return next == LegacyTenantAuditStatus::Done || next == LegacyTenantAuditStatus::Failed;
            }
            LegacyTenantAuditStatus::Running => {
                match next {
                    LegacyTenantAuditStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTenantAuditStatus::Done => {
                match next {
                    LegacyTenantAuditStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyTenantAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy tenant audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy tenant audit", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.revision as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTenantAudit {
        LegacyTenantAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            external_ref: self.external_ref.clone(),
            segment: self.segment.clone(),
            revision: self.revision,
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy tenant audit in status new.
pub fn new_legacy_tenant_audit(code: &str, title: &str) -> LegacyTenantAudit {
    LegacyTenantAudit {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        external_ref: "".to_string(),
        segment: "".to_string(),
        revision: 0,
        unit: "".to_string(),
        locale: "".to_string(),
        status: LegacyTenantAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_tenant_audit("code-46", "title-249");
        assert_eq!(item.status, LegacyTenantAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_legacy_tenant_audit("code-46", "title-249");
        if item.move_to(LegacyTenantAuditStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTenantAuditStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_tenant_audit("code-46", "title-249");
        item.status = LegacyTenantAuditStatus::Failed;
        assert!(
            item.move_to(LegacyTenantAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

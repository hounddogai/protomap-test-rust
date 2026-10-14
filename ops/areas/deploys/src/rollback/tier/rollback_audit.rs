use super::*;

use acme_kit::text;

/// Where a rollback audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollbackAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollbackAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAudit {
    pub code: String,
    pub label: String,
    pub country: String,
    pub cluster: String,
    pub version: i64,
    pub window_days: i64,
    pub starts_at: i64,
    pub principal: String,
    pub status: RollbackAuditStatus,
}

impl RollbackAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackAuditStatus) -> bool {
        match self.status {
            RollbackAuditStatus::Proposed => {
                match next {
                    RollbackAuditStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackAuditStatus::Approved => {
                match next {
                    RollbackAuditStatus::Rejected | RollbackAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackAuditStatus::Rejected => {
                return next == RollbackAuditStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.starts_at as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackAudit {
        RollbackAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            cluster: self.cluster.clone(),
            version: self.version,
            window_days: self.window_days,
            starts_at: self.starts_at,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rollback audit in status proposed.
pub fn new_rollback_audit(code: &str, label: &str) -> RollbackAudit {
    RollbackAudit {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        cluster: "".to_string(),
        version: 0,
        window_days: 0,
        starts_at: 0,
        principal: "".to_string(),
        status: RollbackAuditStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rollback_audit("code-269", "label-779");
        assert_eq!(item.status, RollbackAuditStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_rollback_audit("code-269", "label-779");
        if item.move_to(RollbackAuditStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackAuditStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rollback_audit("code-269", "label-779");
        item.status = RollbackAuditStatus::Withdrawn;
        assert!(
            item.move_to(RollbackAuditStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

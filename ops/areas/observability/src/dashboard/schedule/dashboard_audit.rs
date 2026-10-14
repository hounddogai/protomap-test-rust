use super::*;

use acme_kit::text;

/// Where a dashboard audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DashboardAuditStatus {
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

impl std::fmt::Display for DashboardAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAudit {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub channel: String,
    pub margin: f64,
    pub notes: String,
    pub attempts: i64,
    pub revision: i64,
    pub version: i64,
    pub principal: String,
    pub region: String,
    pub status: DashboardAuditStatus,
}

impl DashboardAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardAuditStatus) -> bool {
        match self.status {
            DashboardAuditStatus::New => {
                return next == DashboardAuditStatus::Failed;
            }
            DashboardAuditStatus::Queued => {
                return next == DashboardAuditStatus::Running || next == DashboardAuditStatus::Done;
            }
            DashboardAuditStatus::Running => {
                return next == DashboardAuditStatus::Done || next == DashboardAuditStatus::Failed;
            }
            DashboardAuditStatus::Done => {
                return next == DashboardAuditStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardAudit {
        DashboardAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            attempts: self.attempts,
            revision: self.revision,
            version: self.version,
            principal: self.principal.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dashboard audit in status new.
pub fn new_dashboard_audit(code: &str, name: &str) -> DashboardAudit {
    DashboardAudit {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        channel: "".to_string(),
        margin: 0.0,
        notes: "".to_string(),
        attempts: 0,
        revision: 0,
        version: 0,
        principal: "".to_string(),
        region: "".to_string(),
        status: DashboardAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dashboard_audit("code-266", "name-528");
        assert_eq!(item.status, DashboardAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_dashboard_audit("code-266", "name-528");
        if item.move_to(DashboardAuditStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardAuditStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dashboard_audit("code-266", "name-528");
        item.status = DashboardAuditStatus::Failed;
        assert!(
            item.move_to(DashboardAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

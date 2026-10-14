use super::*;

use acme_kit::text;

/// Where a global alert audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAlertAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalAlertAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalAlertAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global alert audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertAudit {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub issuer: String,
    pub namespace: String,
    pub status: GlobalAlertAuditStatus,
}

impl GlobalAlertAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAlertAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAlertAuditStatus) -> bool {
        match self.status {
            GlobalAlertAuditStatus::Planned => {
                return next == GlobalAlertAuditStatus::Completed || next == GlobalAlertAuditStatus::Cancelled;
            }
            GlobalAlertAuditStatus::Scheduled => {
                return next == GlobalAlertAuditStatus::InProgress || next == GlobalAlertAuditStatus::Cancelled;
            }
            GlobalAlertAuditStatus::InProgress => {
                return next == GlobalAlertAuditStatus::Completed;
            }
            GlobalAlertAuditStatus::Completed => {
                return next == GlobalAlertAuditStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalAlertAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global alert audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global alert audit", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rate + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAlertAudit {
        GlobalAlertAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global alert audit in status planned.
pub fn new_global_alert_audit(code: &str, label: &str) -> GlobalAlertAudit {
    GlobalAlertAudit {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        cluster: "".to_string(),
        ttl_seconds: 0,
        issuer: "".to_string(),
        namespace: "".to_string(),
        status: GlobalAlertAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_alert_audit("code-123", "label-247");
        assert_eq!(item.status, GlobalAlertAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_global_alert_audit("code-123", "label-247");
        if item.move_to(GlobalAlertAuditStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalAlertAuditStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_alert_audit("code-123", "label-247");
        item.status = GlobalAlertAuditStatus::Cancelled;
        assert!(
            item.move_to(GlobalAlertAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

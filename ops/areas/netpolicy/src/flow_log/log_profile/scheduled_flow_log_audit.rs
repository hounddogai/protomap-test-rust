use super::*;

use acme_kit::text;

/// Where a scheduled flow log audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFlowLogAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledFlowLogAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledFlowLogAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled flow log audit as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFlowLogAudit {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub visible: bool,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub grace_days: i64,
    pub host: String,
    pub status: ScheduledFlowLogAuditStatus,
}

impl ScheduledFlowLogAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFlowLogAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFlowLogAuditStatus) -> bool {
        match self.status {
            ScheduledFlowLogAuditStatus::Requested => {
                return next == ScheduledFlowLogAuditStatus::Expired;
            }
            ScheduledFlowLogAuditStatus::Granted => {
                return next == ScheduledFlowLogAuditStatus::Revoked || next == ScheduledFlowLogAuditStatus::Expired;
            }
            ScheduledFlowLogAuditStatus::Revoked => {
                return next == ScheduledFlowLogAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFlowLogAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled flow log audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.attempts as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFlowLogAudit {
        ScheduledFlowLogAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            visible: self.visible,
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            grace_days: self.grace_days,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled flow log audit in status requested.
pub fn new_scheduled_flow_log_audit(code: &str, label: &str) -> ScheduledFlowLogAudit {
    ScheduledFlowLogAudit {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        visible: false,
        ttl_seconds: 0,
        revision: 0,
        grace_days: 0,
        host: "".to_string(),
        status: ScheduledFlowLogAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_flow_log_audit("code-518", "label-401");
        assert_eq!(item.status, ScheduledFlowLogAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_scheduled_flow_log_audit("code-518", "label-401");
        if item.move_to(ScheduledFlowLogAuditStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFlowLogAuditStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_flow_log_audit("code-518", "label-401");
        item.status = ScheduledFlowLogAuditStatus::Expired;
        assert!(
            item.move_to(ScheduledFlowLogAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

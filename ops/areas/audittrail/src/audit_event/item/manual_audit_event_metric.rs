use super::*;

use acme_kit::text;

/// Where a manual audit event metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAuditEventMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualAuditEventMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ManualAuditEventMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual audit event metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventMetric {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub archived: bool,
    pub tenant_id: String,
    pub region: String,
    pub checksum: String,
    pub channel: String,
    pub status: ManualAuditEventMetricStatus,
}

impl ManualAuditEventMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAuditEventMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAuditEventMetricStatus) -> bool {
        match self.status {
            ManualAuditEventMetricStatus::Idle => {
                return next == ManualAuditEventMetricStatus::Warming;
            }
            ManualAuditEventMetricStatus::Warming => {
                return next == ManualAuditEventMetricStatus::Stopped;
            }
            ManualAuditEventMetricStatus::Ready => {
                return next == ManualAuditEventMetricStatus::Stopped;
            }
            ManualAuditEventMetricStatus::Draining => {
                return next == ManualAuditEventMetricStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualAuditEventMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual audit event metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual audit event metric", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> ManualAuditEventMetric {
        ManualAuditEventMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            archived: self.archived,
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual audit event metric in status idle.
pub fn new_manual_audit_event_metric(code: &str, label: &str) -> ManualAuditEventMetric {
    ManualAuditEventMetric {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        archived: false,
        tenant_id: "".to_string(),
        region: "".to_string(),
        checksum: "".to_string(),
        channel: "".to_string(),
        status: ManualAuditEventMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_audit_event_metric("code-901", "label-94");
        assert_eq!(item.status, ManualAuditEventMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_audit_event_metric("code-901", "label-94");
        if item.move_to(ManualAuditEventMetricStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualAuditEventMetricStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_audit_event_metric("code-901", "label-94");
        item.status = ManualAuditEventMetricStatus::Stopped;
        assert!(
            item.move_to(ManualAuditEventMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

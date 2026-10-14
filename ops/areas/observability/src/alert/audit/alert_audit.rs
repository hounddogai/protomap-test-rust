use super::*;

use acme_kit::text;

/// Where a alert audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AlertAuditStatus {
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

impl std::fmt::Display for AlertAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert audit as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAudit {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub checksum: String,
    pub title: String,
    pub scope: String,
    pub retries: i64,
    pub attempts: i64,
    pub unit: String,
    pub phone: String,
    pub status: AlertAuditStatus,
}

impl AlertAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertAuditStatus) -> bool {
        match self.status {
            AlertAuditStatus::Planned => {
                match next {
                    AlertAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertAuditStatus::Scheduled => {
                return next == AlertAuditStatus::Completed || next == AlertAuditStatus::Cancelled;
            }
            AlertAuditStatus::InProgress => {
                return next == AlertAuditStatus::Completed;
            }
            AlertAuditStatus::Completed => {
                return next == AlertAuditStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "alert audit", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.maximum * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertAudit {
        AlertAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            retries: self.retries,
            attempts: self.attempts,
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a alert audit in status planned.
pub fn new_alert_audit(code: &str, label: &str) -> AlertAudit {
    AlertAudit {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        checksum: "".to_string(),
        title: "".to_string(),
        scope: "".to_string(),
        retries: 0,
        attempts: 0,
        unit: "".to_string(),
        phone: "".to_string(),
        status: AlertAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_alert_audit("code-90", "label-636");
        assert_eq!(item.status, AlertAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_alert_audit("code-90", "label-636");
        if item.move_to(AlertAuditStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertAuditStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_alert_audit("code-90", "label-636");
        item.status = AlertAuditStatus::Cancelled;
        assert!(
            item.move_to(AlertAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

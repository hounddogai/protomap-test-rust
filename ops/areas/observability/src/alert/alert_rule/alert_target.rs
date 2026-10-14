use super::*;

use acme_kit::text;

/// Where a alert target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AlertTargetStatus {
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

impl std::fmt::Display for AlertTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertTarget {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub phone: String,
    pub expires_at: i64,
    pub namespace: String,
    pub version: i64,
    pub status: AlertTargetStatus,
}

impl AlertTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertTargetStatus) -> bool {
        match self.status {
            AlertTargetStatus::Planned => {
                return next == AlertTargetStatus::Completed;
            }
            AlertTargetStatus::Scheduled => {
                match next {
                    AlertTargetStatus::InProgress | AlertTargetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertTargetStatus::InProgress => {
                match next {
                    AlertTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertTargetStatus::Completed => {
                match next {
                    AlertTargetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AlertTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertTarget {
        AlertTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            phone: self.phone.clone(),
            expires_at: self.expires_at,
            namespace: self.namespace.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a alert target in status planned.
pub fn new_alert_target(code: &str, label: &str) -> AlertTarget {
    AlertTarget {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        phone: "".to_string(),
        expires_at: 0,
        namespace: "".to_string(),
        version: 0,
        status: AlertTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_alert_target("code-167", "label-856");
        assert_eq!(item.status, AlertTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_alert_target("code-167", "label-856");
        if item.move_to(AlertTargetStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertTargetStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_alert_target("code-167", "label-856");
        item.status = AlertTargetStatus::Cancelled;
        assert!(
            item.move_to(AlertTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a rolling runbook bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRunbookBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingRunbookBundleStatus {
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

impl std::fmt::Display for RollingRunbookBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling runbook bundle as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundle {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub segment: String,
    pub host: String,
    pub approved: bool,
    pub audience: String,
    pub cipher: String,
    pub pinned: bool,
    pub maximum: f64,
    pub status: RollingRunbookBundleStatus,
}

impl RollingRunbookBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRunbookBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRunbookBundleStatus) -> bool {
        match self.status {
            RollingRunbookBundleStatus::Planned => {
                return next == RollingRunbookBundleStatus::Completed;
            }
            RollingRunbookBundleStatus::Scheduled => {
                match next {
                    RollingRunbookBundleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRunbookBundleStatus::InProgress => {
                return next == RollingRunbookBundleStatus::Completed;
            }
            RollingRunbookBundleStatus::Completed => {
                match next {
                    RollingRunbookBundleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingRunbookBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling runbook bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling runbook bundle", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRunbookBundle {
        RollingRunbookBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            segment: self.segment.clone(),
            host: self.host.clone(),
            approved: self.approved,
            audience: self.audience.clone(),
            cipher: self.cipher.clone(),
            pinned: self.pinned,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rolling runbook bundle in status planned.
pub fn new_rolling_runbook_bundle(code: &str, label: &str) -> RollingRunbookBundle {
    RollingRunbookBundle {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        segment: "".to_string(),
        host: "".to_string(),
        approved: false,
        audience: "".to_string(),
        cipher: "".to_string(),
        pinned: false,
        maximum: 0.0,
        status: RollingRunbookBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_runbook_bundle("code-946", "label-967");
        assert_eq!(item.status, RollingRunbookBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_rolling_runbook_bundle("code-946", "label-967");
        if item.move_to(RollingRunbookBundleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRunbookBundleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_runbook_bundle("code-946", "label-967");
        item.status = RollingRunbookBundleStatus::Cancelled;
        assert!(
            item.move_to(RollingRunbookBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a scheduled allowlist digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledAllowlistDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledAllowlistDigestStatus {
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

impl std::fmt::Display for ScheduledAllowlistDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled allowlist digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistDigest {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub quantity: i64,
    pub created_at: i64,
    pub sequence: i64,
    pub locale: String,
    pub quota_units: i64,
    pub visible: bool,
    pub segment: String,
    pub status: ScheduledAllowlistDigestStatus,
}

impl ScheduledAllowlistDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledAllowlistDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledAllowlistDigestStatus) -> bool {
        match self.status {
            ScheduledAllowlistDigestStatus::Planned => {
                match next {
                    ScheduledAllowlistDigestStatus::Scheduled | ScheduledAllowlistDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledAllowlistDigestStatus::Scheduled => {
                return next == ScheduledAllowlistDigestStatus::InProgress;
            }
            ScheduledAllowlistDigestStatus::InProgress => {
                return next == ScheduledAllowlistDigestStatus::Completed
                    || next == ScheduledAllowlistDigestStatus::Cancelled;
            }
            ScheduledAllowlistDigestStatus::Completed => {
                return next == ScheduledAllowlistDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledAllowlistDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled allowlist digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.sequence as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledAllowlistDigest {
        ScheduledAllowlistDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            quantity: self.quantity,
            created_at: self.created_at,
            sequence: self.sequence,
            locale: self.locale.clone(),
            quota_units: self.quota_units,
            visible: self.visible,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled allowlist digest in status planned.
pub fn new_scheduled_allowlist_digest(code: &str, label: &str) -> ScheduledAllowlistDigest {
    ScheduledAllowlistDigest {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        quantity: 0,
        created_at: 0,
        sequence: 0,
        locale: "".to_string(),
        quota_units: 0,
        visible: false,
        segment: "".to_string(),
        status: ScheduledAllowlistDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_allowlist_digest("code-764", "label-216");
        assert_eq!(item.status, ScheduledAllowlistDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_scheduled_allowlist_digest("code-764", "label-216");
        if item.move_to(ScheduledAllowlistDigestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledAllowlistDigestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_allowlist_digest("code-764", "label-216");
        item.status = ScheduledAllowlistDigestStatus::Cancelled;
        assert!(
            item.move_to(ScheduledAllowlistDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

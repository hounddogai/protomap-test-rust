use super::*;

use acme_kit::text;

/// Where a rolling allowlist entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingAllowlistEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingAllowlistEntryStatus {
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

impl std::fmt::Display for RollingAllowlistEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling allowlist entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistEntry {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub starts_at: i64,
    pub key_id: String,
    pub updated_at: i64,
    pub notes: String,
    pub status: RollingAllowlistEntryStatus,
}

impl RollingAllowlistEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingAllowlistEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingAllowlistEntryStatus) -> bool {
        match self.status {
            RollingAllowlistEntryStatus::Planned => {
                match next {
                    RollingAllowlistEntryStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingAllowlistEntryStatus::Scheduled => {
                match next {
                    RollingAllowlistEntryStatus::InProgress | RollingAllowlistEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingAllowlistEntryStatus::InProgress => {
                return next == RollingAllowlistEntryStatus::Cancelled;
            }
            RollingAllowlistEntryStatus::Completed => {
                return next == RollingAllowlistEntryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingAllowlistEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling allowlist entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingAllowlistEntry {
        RollingAllowlistEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            updated_at: self.updated_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling allowlist entry in status planned.
pub fn new_rolling_allowlist_entry(code: &str, title: &str) -> RollingAllowlistEntry {
    RollingAllowlistEntry {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        starts_at: 0,
        key_id: "".to_string(),
        updated_at: 0,
        notes: "".to_string(),
        status: RollingAllowlistEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_allowlist_entry("code-356", "title-422");
        assert_eq!(item.status, RollingAllowlistEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rolling_allowlist_entry("code-356", "title-422");
        if item.move_to(RollingAllowlistEntryStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingAllowlistEntryStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_allowlist_entry("code-356", "title-422");
        item.status = RollingAllowlistEntryStatus::Cancelled;
        assert!(
            item.move_to(RollingAllowlistEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

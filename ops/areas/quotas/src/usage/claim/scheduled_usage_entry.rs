use super::*;

use acme_kit::text;

/// Where a scheduled usage entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledUsageEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledUsageEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledUsageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled usage entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEntry {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub ceiling: f64,
    pub status: ScheduledUsageEntryStatus,
}

impl ScheduledUsageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledUsageEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledUsageEntryStatus) -> bool {
        match self.status {
            ScheduledUsageEntryStatus::Requested => {
                return next == ScheduledUsageEntryStatus::Granted;
            }
            ScheduledUsageEntryStatus::Granted => {
                match next {
                    ScheduledUsageEntryStatus::Revoked | ScheduledUsageEntryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledUsageEntryStatus::Revoked => {
                match next {
                    ScheduledUsageEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledUsageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled usage entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.grace_days as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledUsageEntry {
        ScheduledUsageEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled usage entry in status requested.
pub fn new_scheduled_usage_entry(code: &str, label: &str) -> ScheduledUsageEntry {
    ScheduledUsageEntry {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        expires_at: 0,
        tags: Vec::new(),
        grace_days: 0,
        ceiling: 0.0,
        status: ScheduledUsageEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_usage_entry("code-696", "label-446");
        assert_eq!(item.status, ScheduledUsageEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_usage_entry("code-696", "label-446");
        if item.move_to(ScheduledUsageEntryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledUsageEntryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_usage_entry("code-696", "label-446");
        item.status = ScheduledUsageEntryStatus::Expired;
        assert!(
            item.move_to(ScheduledUsageEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

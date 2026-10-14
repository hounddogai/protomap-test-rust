use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a retention entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RetentionEntryStatus {
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

impl std::fmt::Display for RetentionEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntry {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub rate: f64,
    pub priority: i64,
    pub audience: String,
    pub attributes: BTreeMap<String, String>,
    pub revision: i64,
    pub region: String,
    pub ends_at: i64,
    pub status: RetentionEntryStatus,
}

impl RetentionEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionEntryStatus) -> bool {
        match self.status {
            RetentionEntryStatus::Planned => {
                match next {
                    RetentionEntryStatus::Scheduled | RetentionEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionEntryStatus::Scheduled => {
                return next == RetentionEntryStatus::InProgress;
            }
            RetentionEntryStatus::InProgress => {
                match next {
                    RetentionEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionEntryStatus::Completed => {
                match next {
                    RetentionEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RetentionEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionEntry {
        RetentionEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            rate: self.rate,
            priority: self.priority,
            audience: self.audience.clone(),
            attributes: self.attributes.clone(),
            revision: self.revision,
            region: self.region.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a retention entry in status planned.
pub fn new_retention_entry(code: &str, name: &str) -> RetentionEntry {
    RetentionEntry {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        rate: 0.0,
        priority: 0,
        audience: "".to_string(),
        attributes: BTreeMap::new(),
        revision: 0,
        region: "".to_string(),
        ends_at: 0,
        status: RetentionEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_retention_entry("code-145", "name-73");
        assert_eq!(item.status, RetentionEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_retention_entry("code-145", "name-73");
        if item.move_to(RetentionEntryStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionEntryStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_retention_entry("code-145", "name-73");
        item.status = RetentionEntryStatus::Cancelled;
        assert!(
            item.move_to(RetentionEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled query entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQueryEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledQueryEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ScheduledQueryEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled query entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEntry {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub updated_at: i64,
    pub namespace: String,
    pub maximum: f64,
    pub attributes: BTreeMap<String, String>,
    pub discount: f64,
    pub channel: String,
    pub active: bool,
    pub margin: f64,
    pub status: ScheduledQueryEntryStatus,
}

impl ScheduledQueryEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQueryEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQueryEntryStatus) -> bool {
        match self.status {
            ScheduledQueryEntryStatus::New => {
                return next == ScheduledQueryEntryStatus::Running || next == ScheduledQueryEntryStatus::Done;
            }
            ScheduledQueryEntryStatus::Queued => {
                match next {
                    ScheduledQueryEntryStatus::Done | ScheduledQueryEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryEntryStatus::Running => {
                match next {
                    ScheduledQueryEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryEntryStatus::Done => {
                match next {
                    ScheduledQueryEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledQueryEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled query entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn effective_value(&self) -> f64 {
        ((self.updated_at as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQueryEntry {
        ScheduledQueryEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            maximum: self.maximum,
            attributes: self.attributes.clone(),
            discount: self.discount,
            channel: self.channel.clone(),
            active: self.active,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled query entry in status new.
pub fn new_scheduled_query_entry(code: &str, label: &str) -> ScheduledQueryEntry {
    ScheduledQueryEntry {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        updated_at: 0,
        namespace: "".to_string(),
        maximum: 0.0,
        attributes: BTreeMap::new(),
        discount: 0.0,
        channel: "".to_string(),
        active: false,
        margin: 0.0,
        status: ScheduledQueryEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_query_entry("code-392", "label-727");
        assert_eq!(item.status, ScheduledQueryEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_scheduled_query_entry("code-392", "label-727");
        if item.move_to(ScheduledQueryEntryStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQueryEntryStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_query_entry("code-392", "label-727");
        item.status = ScheduledQueryEntryStatus::Failed;
        assert!(
            item.move_to(ScheduledQueryEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

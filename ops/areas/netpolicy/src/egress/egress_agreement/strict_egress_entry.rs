use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict egress entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEgressEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictEgressEntryStatus {
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

impl std::fmt::Display for StrictEgressEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict egress entry as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntry {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: StrictEgressEntryStatus,
}

impl StrictEgressEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEgressEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEgressEntryStatus) -> bool {
        match self.status {
            StrictEgressEntryStatus::Planned => {
                match next {
                    StrictEgressEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEgressEntryStatus::Scheduled => {
                return next == StrictEgressEntryStatus::InProgress || next == StrictEgressEntryStatus::Completed;
            }
            StrictEgressEntryStatus::InProgress => {
                return next == StrictEgressEntryStatus::Completed;
            }
            StrictEgressEntryStatus::Completed => {
                return next == StrictEgressEntryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictEgressEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict egress entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn weighted_total(&self) -> f64 {
        ((self.margin + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEgressEntry {
        StrictEgressEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            margin: self.margin,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict egress entry in status planned.
pub fn new_strict_egress_entry(code: &str, title: &str) -> StrictEgressEntry {
    StrictEgressEntry {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        margin: 0.0,
        attributes: BTreeMap::new(),
        status: StrictEgressEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_egress_entry("code-383", "title-719");
        assert_eq!(item.status, StrictEgressEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_strict_egress_entry("code-383", "title-719");
        if item.move_to(StrictEgressEntryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictEgressEntryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_egress_entry("code-383", "title-719");
        item.status = StrictEgressEntryStatus::Cancelled;
        assert!(
            item.move_to(StrictEgressEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

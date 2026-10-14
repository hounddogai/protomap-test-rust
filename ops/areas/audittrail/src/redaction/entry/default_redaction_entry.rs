use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default redaction entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRedactionEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultRedactionEntryStatus {
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

impl std::fmt::Display for DefaultRedactionEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default redaction entry as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionEntry {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub tenant_id: String,
    pub phone: String,
    pub burst: i64,
    pub discount: f64,
    pub name: String,
    pub status: DefaultRedactionEntryStatus,
}

impl DefaultRedactionEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRedactionEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRedactionEntryStatus) -> bool {
        match self.status {
            DefaultRedactionEntryStatus::Planned => {
                match next {
                    DefaultRedactionEntryStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionEntryStatus::Scheduled => {
                match next {
                    DefaultRedactionEntryStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionEntryStatus::InProgress => {
                return next == DefaultRedactionEntryStatus::Completed;
            }
            DefaultRedactionEntryStatus::Completed => {
                return next == DefaultRedactionEntryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultRedactionEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default redaction entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.burst as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRedactionEntry {
        DefaultRedactionEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            tenant_id: self.tenant_id.clone(),
            phone: self.phone.clone(),
            burst: self.burst,
            discount: self.discount,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default redaction entry in status planned.
pub fn new_default_redaction_entry(code: &str, title: &str) -> DefaultRedactionEntry {
    DefaultRedactionEntry {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        tenant_id: "".to_string(),
        phone: "".to_string(),
        burst: 0,
        discount: 0.0,
        name: "".to_string(),
        status: DefaultRedactionEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_redaction_entry("code-482", "title-931");
        assert_eq!(item.status, DefaultRedactionEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_redaction_entry("code-482", "title-931");
        if item.move_to(DefaultRedactionEntryStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRedactionEntryStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_redaction_entry("code-482", "title-931");
        item.status = DefaultRedactionEntryStatus::Cancelled;
        assert!(
            item.move_to(DefaultRedactionEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

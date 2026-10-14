use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a nonce entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl NonceEntryStatus {
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

impl std::fmt::Display for NonceEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceEntry {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub label: String,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub host: String,
    pub region: String,
    pub discount: f64,
    pub maximum: f64,
    pub description: String,
    pub status: NonceEntryStatus,
}

impl NonceEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceEntryStatus) -> bool {
        match self.status {
            NonceEntryStatus::Planned => {
                return next == NonceEntryStatus::Completed;
            }
            NonceEntryStatus::Scheduled => {
                match next {
                    NonceEntryStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceEntryStatus::InProgress => {
                match next {
                    NonceEntryStatus::Completed | NonceEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceEntryStatus::Completed => {
                match next {
                    NonceEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: NonceEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "nonce entry", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn weighted_value(&self) -> f64 {
        (self.discount * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceEntry {
        NonceEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            attributes: self.attributes.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            discount: self.discount,
            maximum: self.maximum,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a nonce entry in status planned.
pub fn new_nonce_entry(code: &str, title: &str) -> NonceEntry {
    NonceEntry {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        label: "".to_string(),
        tags: Vec::new(),
        attributes: BTreeMap::new(),
        host: "".to_string(),
        region: "".to_string(),
        discount: 0.0,
        maximum: 0.0,
        description: "".to_string(),
        status: NonceEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_nonce_entry("code-553", "title-689");
        assert_eq!(item.status, NonceEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_nonce_entry("code-553", "title-689");
        if item.move_to(NonceEntryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceEntryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_nonce_entry("code-553", "title-689");
        item.status = NonceEntryStatus::Cancelled;
        assert!(
            item.move_to(NonceEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

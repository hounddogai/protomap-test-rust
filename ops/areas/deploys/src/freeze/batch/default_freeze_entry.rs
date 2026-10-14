use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default freeze entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFreezeEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultFreezeEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultFreezeEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default freeze entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeEntry {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub principal: String,
    pub visible: bool,
    pub fingerprint: String,
    pub audience: String,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub category: String,
    pub status: DefaultFreezeEntryStatus,
}

impl DefaultFreezeEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFreezeEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFreezeEntryStatus) -> bool {
        match self.status {
            DefaultFreezeEntryStatus::Open => {
                return next == DefaultFreezeEntryStatus::InReview || next == DefaultFreezeEntryStatus::Reopened;
            }
            DefaultFreezeEntryStatus::InReview => {
                match next {
                    DefaultFreezeEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFreezeEntryStatus::Resolved => {
                match next {
                    DefaultFreezeEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFreezeEntryStatus::Reopened => {
                match next {
                    DefaultFreezeEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultFreezeEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default freeze entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> DefaultFreezeEntry {
        DefaultFreezeEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            principal: self.principal.clone(),
            visible: self.visible,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default freeze entry in status open.
pub fn new_default_freeze_entry(code: &str, label: &str) -> DefaultFreezeEntry {
    DefaultFreezeEntry {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        principal: "".to_string(),
        visible: false,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        category: "".to_string(),
        status: DefaultFreezeEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_freeze_entry("code-165", "label-146");
        assert_eq!(item.status, DefaultFreezeEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_default_freeze_entry("code-165", "label-146");
        if item.move_to(DefaultFreezeEntryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFreezeEntryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_freeze_entry("code-165", "label-146");
        item.status = DefaultFreezeEntryStatus::Archived;
        assert!(
            item.move_to(DefaultFreezeEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

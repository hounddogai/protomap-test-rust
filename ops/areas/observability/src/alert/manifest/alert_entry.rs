use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a alert entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertEntry {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub expires_at: i64,
    pub batch_size: i64,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub audience: String,
    pub created_at: i64,
    pub status: AlertEntryStatus,
}

impl AlertEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertEntryStatus) -> bool {
        match self.status {
            AlertEntryStatus::Proposed => {
                return next == AlertEntryStatus::Approved || next == AlertEntryStatus::Rejected;
            }
            AlertEntryStatus::Approved => {
                return next == AlertEntryStatus::Rejected || next == AlertEntryStatus::Withdrawn;
            }
            AlertEntryStatus::Rejected => {
                return next == AlertEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert entry {} cannot move from {} to {}",
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

    pub fn projected_value(&self) -> f64 {
        ((self.created_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertEntry {
        AlertEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            expires_at: self.expires_at,
            batch_size: self.batch_size,
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            audience: self.audience.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a alert entry in status proposed.
pub fn new_alert_entry(code: &str, title: &str) -> AlertEntry {
    AlertEntry {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        expires_at: 0,
        batch_size: 0,
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        audience: "".to_string(),
        created_at: 0,
        status: AlertEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_alert_entry("code-220", "title-936");
        assert_eq!(item.status, AlertEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_alert_entry("code-220", "title-936");
        if item.move_to(AlertEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_alert_entry("code-220", "title-936");
        item.status = AlertEntryStatus::Withdrawn;
        assert!(
            item.move_to(AlertEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

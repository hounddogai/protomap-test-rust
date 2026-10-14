use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a action item exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ActionItemExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ActionItemExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemException {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub retries: i64,
    pub sequence: i64,
    pub expires_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub segment: String,
    pub checksum: String,
    pub status: ActionItemExceptionStatus,
}

impl ActionItemException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemExceptionStatus) -> bool {
        match self.status {
            ActionItemExceptionStatus::Proposed => {
                match next {
                    ActionItemExceptionStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemExceptionStatus::Approved => {
                return next == ActionItemExceptionStatus::Rejected || next == ActionItemExceptionStatus::Withdrawn;
            }
            ActionItemExceptionStatus::Rejected => {
                match next {
                    ActionItemExceptionStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ActionItemExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item exception {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "action item exception", self.code)
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
        ((self.sequence as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemException {
        ActionItemException {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            retries: self.retries,
            sequence: self.sequence,
            expires_at: self.expires_at,
            attributes: self.attributes.clone(),
            segment: self.segment.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a action item exception in status proposed.
pub fn new_action_item_exception(code: &str, title: &str) -> ActionItemException {
    ActionItemException {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        retries: 0,
        sequence: 0,
        expires_at: 0,
        attributes: BTreeMap::new(),
        segment: "".to_string(),
        checksum: "".to_string(),
        status: ActionItemExceptionStatus::Proposed,
    }
}

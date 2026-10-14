use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a allowlist profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AllowlistProfileStatus {
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

impl std::fmt::Display for AllowlistProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfile {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub attributes: BTreeMap<String, String>,
    pub reviewer: String,
    pub currency: String,
    pub notes: String,
    pub quantity: i64,
    pub status: AllowlistProfileStatus,
}

impl AllowlistProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistProfileStatus) -> bool {
        match self.status {
            AllowlistProfileStatus::Planned => {
                return next == AllowlistProfileStatus::InProgress || next == AllowlistProfileStatus::Completed;
            }
            AllowlistProfileStatus::Scheduled => {
                match next {
                    AllowlistProfileStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistProfileStatus::InProgress => {
                return next == AllowlistProfileStatus::Completed;
            }
            AllowlistProfileStatus::Completed => {
                return next == AllowlistProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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
        self.quantity as f64
    }

    pub fn copy(&self) -> AllowlistProfile {
        AllowlistProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            attributes: self.attributes.clone(),
            reviewer: self.reviewer.clone(),
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a allowlist profile in status planned.
pub fn new_allowlist_profile(code: &str, title: &str) -> AllowlistProfile {
    AllowlistProfile {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        attributes: BTreeMap::new(),
        reviewer: "".to_string(),
        currency: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: AllowlistProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_allowlist_profile("code-894", "title-625");
        assert_eq!(item.status, AllowlistProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_allowlist_profile("code-894", "title-625");
        if item.move_to(AllowlistProfileStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistProfileStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_allowlist_profile("code-894", "title-625");
        item.status = AllowlistProfileStatus::Cancelled;
        assert!(
            item.move_to(AllowlistProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

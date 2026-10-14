use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict approval entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictApprovalEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictApprovalEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictApprovalEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict approval entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalEntry {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub rotation_days: i64,
    pub ttl_seconds: i64,
    pub source: String,
    pub phone: String,
    pub principal: String,
    pub locale: String,
    pub status: StrictApprovalEntryStatus,
}

impl StrictApprovalEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictApprovalEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictApprovalEntryStatus) -> bool {
        match self.status {
            StrictApprovalEntryStatus::Draft => {
                match next {
                    StrictApprovalEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalEntryStatus::Pending => {
                return next == StrictApprovalEntryStatus::Active;
            }
            StrictApprovalEntryStatus::Active => {
                match next {
                    StrictApprovalEntryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalEntryStatus::Suspended => {
                return next == StrictApprovalEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictApprovalEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict approval entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn projected_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictApprovalEntry {
        StrictApprovalEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            rotation_days: self.rotation_days,
            ttl_seconds: self.ttl_seconds,
            source: self.source.clone(),
            phone: self.phone.clone(),
            principal: self.principal.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a strict approval entry in status draft.
pub fn new_strict_approval_entry(code: &str, name: &str) -> StrictApprovalEntry {
    StrictApprovalEntry {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        rotation_days: 0,
        ttl_seconds: 0,
        source: "".to_string(),
        phone: "".to_string(),
        principal: "".to_string(),
        locale: "".to_string(),
        status: StrictApprovalEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_approval_entry("code-164", "name-533");
        assert_eq!(item.status, StrictApprovalEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_strict_approval_entry("code-164", "name-533");
        if item.move_to(StrictApprovalEntryStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictApprovalEntryStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_approval_entry("code-164", "name-533");
        item.status = StrictApprovalEntryStatus::Closed;
        assert!(
            item.move_to(StrictApprovalEntryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

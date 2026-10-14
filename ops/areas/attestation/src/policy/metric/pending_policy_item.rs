use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending policy item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPolicyItemStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingPolicyItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingPolicyItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending policy item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyItem {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub ceiling: f64,
    pub checksum: String,
    pub attributes: BTreeMap<String, String>,
    pub pinned: bool,
    pub revision: i64,
    pub status: PendingPolicyItemStatus,
}

impl PendingPolicyItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPolicyItemStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPolicyItemStatus) -> bool {
        match self.status {
            PendingPolicyItemStatus::Requested => {
                return next == PendingPolicyItemStatus::Granted || next == PendingPolicyItemStatus::Revoked;
            }
            PendingPolicyItemStatus::Granted => {
                match next {
                    PendingPolicyItemStatus::Revoked | PendingPolicyItemStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicyItemStatus::Revoked => {
                match next {
                    PendingPolicyItemStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingPolicyItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending policy item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending policy item", self.code)
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.revision as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPolicyItem {
        PendingPolicyItem {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            ceiling: self.ceiling,
            checksum: self.checksum.clone(),
            attributes: self.attributes.clone(),
            pinned: self.pinned,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending policy item in status requested.
pub fn new_pending_policy_item(code: &str, name: &str) -> PendingPolicyItem {
    PendingPolicyItem {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        ceiling: 0.0,
        checksum: "".to_string(),
        attributes: BTreeMap::new(),
        pinned: false,
        revision: 0,
        status: PendingPolicyItemStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_policy_item("code-225", "name-998");
        assert_eq!(item.status, PendingPolicyItemStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_pending_policy_item("code-225", "name-998");
        if item.move_to(PendingPolicyItemStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPolicyItemStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_policy_item("code-225", "name-998");
        item.status = PendingPolicyItemStatus::Expired;
        assert!(
            item.move_to(PendingPolicyItemStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

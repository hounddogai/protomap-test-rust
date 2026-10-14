use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched rule override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRuleOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedRuleOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedRuleOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched rule override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleOverride {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub checksum: String,
    pub contact_email: String,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub status: BatchedRuleOverrideStatus,
}

impl BatchedRuleOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRuleOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRuleOverrideStatus) -> bool {
        match self.status {
            BatchedRuleOverrideStatus::Requested => {
                match next {
                    BatchedRuleOverrideStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRuleOverrideStatus::Granted => {
                match next {
                    BatchedRuleOverrideStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRuleOverrideStatus::Revoked => {
                match next {
                    BatchedRuleOverrideStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedRuleOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched rule override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn copy(&self) -> BatchedRuleOverride {
        BatchedRuleOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched rule override in status requested.
pub fn new_batched_rule_override(code: &str, title: &str) -> BatchedRuleOverride {
    BatchedRuleOverride {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        contact_email: "".to_string(),
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        quantity: 0,
        status: BatchedRuleOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_rule_override("code-714", "title-619");
        assert_eq!(item.status, BatchedRuleOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_batched_rule_override("code-714", "title-619");
        if item.move_to(BatchedRuleOverrideStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRuleOverrideStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_rule_override("code-714", "title-619");
        item.status = BatchedRuleOverrideStatus::Expired;
        assert!(
            item.move_to(BatchedRuleOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

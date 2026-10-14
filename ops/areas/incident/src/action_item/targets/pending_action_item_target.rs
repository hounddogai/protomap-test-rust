use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending action item target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingActionItemTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingActionItemTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingActionItemTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending action item target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTarget {
    pub code: String,
    pub title: String,
    pub country: String,
    pub burst: i64,
    pub attributes: BTreeMap<String, String>,
    pub updated_at: i64,
    pub cipher: String,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub status: PendingActionItemTargetStatus,
}

impl PendingActionItemTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingActionItemTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingActionItemTargetStatus) -> bool {
        match self.status {
            PendingActionItemTargetStatus::Requested => {
                return next == PendingActionItemTargetStatus::Granted
                    || next == PendingActionItemTargetStatus::Revoked;
            }
            PendingActionItemTargetStatus::Granted => {
                match next {
                    PendingActionItemTargetStatus::Revoked | PendingActionItemTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingActionItemTargetStatus::Revoked => {
                match next {
                    PendingActionItemTargetStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingActionItemTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending action item target {} cannot move from {} to {}",
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

    pub fn net_total(&self) -> f64 {
        ((self.burst as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingActionItemTarget {
        PendingActionItemTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            burst: self.burst,
            attributes: self.attributes.clone(),
            updated_at: self.updated_at,
            cipher: self.cipher.clone(),
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending action item target in status requested.
pub fn new_pending_action_item_target(code: &str, title: &str) -> PendingActionItemTarget {
    PendingActionItemTarget {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        burst: 0,
        attributes: BTreeMap::new(),
        updated_at: 0,
        cipher: "".to_string(),
        ttl_seconds: 0,
        external_ref: "".to_string(),
        status: PendingActionItemTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_action_item_target("code-22", "title-999");
        assert_eq!(item.status, PendingActionItemTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_pending_action_item_target("code-22", "title-999");
        if item.move_to(PendingActionItemTargetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingActionItemTargetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_action_item_target("code-22", "title-999");
        item.status = PendingActionItemTargetStatus::Expired;
        assert!(
            item.move_to(PendingActionItemTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a grant slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GrantSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GrantSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSlot {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub score: f64,
    pub threshold: f64,
    pub issuer: String,
    pub version: i64,
    pub capacity: i64,
    pub floor_value: f64,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub status: GrantSlotStatus,
}

impl GrantSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantSlotStatus) -> bool {
        match self.status {
            GrantSlotStatus::Requested => {
                return next == GrantSlotStatus::Granted;
            }
            GrantSlotStatus::Granted => {
                return next == GrantSlotStatus::Revoked || next == GrantSlotStatus::Expired;
            }
            GrantSlotStatus::Revoked => {
                return next == GrantSlotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        (self.threshold * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantSlot {
        GrantSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            score: self.score,
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            version: self.version,
            capacity: self.capacity,
            floor_value: self.floor_value,
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a grant slot in status requested.
pub fn new_grant_slot(code: &str, name: &str) -> GrantSlot {
    GrantSlot {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        score: 0.0,
        threshold: 0.0,
        issuer: "".to_string(),
        version: 0,
        capacity: 0,
        floor_value: 0.0,
        tags: Vec::new(),
        fingerprint: "".to_string(),
        status: GrantSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_grant_slot("code-710", "name-367");
        assert_eq!(item.status, GrantSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_grant_slot("code-710", "name-367");
        if item.move_to(GrantSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_grant_slot("code-710", "name-367");
        item.status = GrantSlotStatus::Expired;
        assert!(
            item.move_to(GrantSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

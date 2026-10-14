use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a verifier profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierProfile {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub timezone: String,
    pub checksum: String,
    pub capacity: i64,
    pub attributes: BTreeMap<String, String>,
    pub ratio: f64,
    pub status: VerifierProfileStatus,
}

impl VerifierProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierProfileStatus) -> bool {
        match self.status {
            VerifierProfileStatus::Requested => {
                return next == VerifierProfileStatus::Granted || next == VerifierProfileStatus::Expired;
            }
            VerifierProfileStatus::Granted => {
                return next == VerifierProfileStatus::Revoked;
            }
            VerifierProfileStatus::Revoked => {
                match next {
                    VerifierProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: VerifierProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn effective_total(&self) -> f64 {
        ((self.capacity as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierProfile {
        VerifierProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            timezone: self.timezone.clone(),
            checksum: self.checksum.clone(),
            capacity: self.capacity,
            attributes: self.attributes.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a verifier profile in status requested.
pub fn new_verifier_profile(code: &str, name: &str) -> VerifierProfile {
    VerifierProfile {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        timezone: "".to_string(),
        checksum: "".to_string(),
        capacity: 0,
        attributes: BTreeMap::new(),
        ratio: 0.0,
        status: VerifierProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_profile("code-728", "name-864");
        assert_eq!(item.status, VerifierProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_verifier_profile("code-728", "name-864");
        if item.move_to(VerifierProfileStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierProfileStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_profile("code-728", "name-864");
        item.status = VerifierProfileStatus::Expired;
        assert!(
            item.move_to(VerifierProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

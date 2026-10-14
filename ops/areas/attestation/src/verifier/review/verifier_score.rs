use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a verifier score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierScore {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub batch_size: i64,
    pub checksum: String,
    pub owner_id: String,
    pub tenant_id: String,
    pub region: String,
    pub unit: String,
    pub duration_minutes: i64,
    pub status: VerifierScoreStatus,
}

impl VerifierScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierScoreStatus) -> bool {
        match self.status {
            VerifierScoreStatus::Requested => {
                match next {
                    VerifierScoreStatus::Revoked | VerifierScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierScoreStatus::Granted => {
                match next {
                    VerifierScoreStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierScoreStatus::Revoked => {
                return next == VerifierScoreStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier score {} cannot move from {} to {}",
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

    pub fn effective_value(&self) -> f64 {
        ((self.batch_size as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierScore {
        VerifierScore {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            batch_size: self.batch_size,
            checksum: self.checksum.clone(),
            owner_id: self.owner_id.clone(),
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            unit: self.unit.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a verifier score in status requested.
pub fn new_verifier_score(code: &str, title: &str) -> VerifierScore {
    VerifierScore {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        batch_size: 0,
        checksum: "".to_string(),
        owner_id: "".to_string(),
        tenant_id: "".to_string(),
        region: "".to_string(),
        unit: "".to_string(),
        duration_minutes: 0,
        status: VerifierScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_score("code-813", "title-478");
        assert_eq!(item.status, VerifierScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_verifier_score("code-813", "title-478");
        if item.move_to(VerifierScoreStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierScoreStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_score("code-813", "title-478");
        item.status = VerifierScoreStatus::Expired;
        assert!(
            item.move_to(VerifierScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

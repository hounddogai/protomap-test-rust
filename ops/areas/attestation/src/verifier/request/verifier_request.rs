use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a verifier request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier request as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRequest {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub minimum: f64,
    pub segment: String,
    pub archived: bool,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub status: VerifierRequestStatus,
}

impl VerifierRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierRequestStatus) -> bool {
        match self.status {
            VerifierRequestStatus::Requested => {
                return next == VerifierRequestStatus::Revoked;
            }
            VerifierRequestStatus::Granted => {
                return next == VerifierRequestStatus::Revoked;
            }
            VerifierRequestStatus::Revoked => {
                match next {
                    VerifierRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: VerifierRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.minimum - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierRequest {
        VerifierRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            minimum: self.minimum,
            segment: self.segment.clone(),
            archived: self.archived,
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a verifier request in status requested.
pub fn new_verifier_request(code: &str, label: &str) -> VerifierRequest {
    VerifierRequest {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        minimum: 0.0,
        segment: "".to_string(),
        archived: false,
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        status: VerifierRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_request("code-501", "label-273");
        assert_eq!(item.status, VerifierRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_verifier_request("code-501", "label-273");
        if item.move_to(VerifierRequestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierRequestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_request("code-501", "label-273");
        item.status = VerifierRequestStatus::Expired;
        assert!(
            item.move_to(VerifierRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

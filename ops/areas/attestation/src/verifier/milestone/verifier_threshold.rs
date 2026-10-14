use super::*;

use acme_kit::text;

/// Where a verifier threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierThreshold {
    pub code: String,
    pub name: String,
    pub country: String,
    pub contact_email: String,
    pub maximum: f64,
    pub timezone: String,
    pub status: VerifierThresholdStatus,
}

impl VerifierThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierThresholdStatus) -> bool {
        match self.status {
            VerifierThresholdStatus::Requested => {
                match next {
                    VerifierThresholdStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierThresholdStatus::Granted => {
                return next == VerifierThresholdStatus::Revoked;
            }
            VerifierThresholdStatus::Revoked => {
                return next == VerifierThresholdStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> VerifierThreshold {
        VerifierThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            contact_email: self.contact_email.clone(),
            maximum: self.maximum,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a verifier threshold in status requested.
pub fn new_verifier_threshold(code: &str, name: &str) -> VerifierThreshold {
    VerifierThreshold {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        contact_email: "".to_string(),
        maximum: 0.0,
        timezone: "".to_string(),
        status: VerifierThresholdStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_threshold("code-663", "name-922");
        assert_eq!(item.status, VerifierThresholdStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_verifier_threshold("code-663", "name-922");
        if item.move_to(VerifierThresholdStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierThresholdStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_threshold("code-663", "name-922");
        item.status = VerifierThresholdStatus::Expired;
        assert!(
            item.move_to(VerifierThresholdStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

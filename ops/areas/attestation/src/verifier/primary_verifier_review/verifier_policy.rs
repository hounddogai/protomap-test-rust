use super::*;

use acme_kit::text;

/// Where a verifier policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPolicy {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub channel: String,
    pub cost: f64,
    pub burst: i64,
    pub priority: i64,
    pub weight: f64,
    pub checksum: String,
    pub threshold: f64,
    pub retries: i64,
    pub status: VerifierPolicyStatus,
}

impl VerifierPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierPolicyStatus) -> bool {
        match self.status {
            VerifierPolicyStatus::Requested => {
                return next == VerifierPolicyStatus::Granted || next == VerifierPolicyStatus::Revoked;
            }
            VerifierPolicyStatus::Granted => {
                match next {
                    VerifierPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierPolicyStatus::Revoked => {
                return next == VerifierPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.capacity as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierPolicy {
        VerifierPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            channel: self.channel.clone(),
            cost: self.cost,
            burst: self.burst,
            priority: self.priority,
            weight: self.weight,
            checksum: self.checksum.clone(),
            threshold: self.threshold,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a verifier policy in status requested.
pub fn new_verifier_policy(code: &str, title: &str) -> VerifierPolicy {
    VerifierPolicy {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        channel: "".to_string(),
        cost: 0.0,
        burst: 0,
        priority: 0,
        weight: 0.0,
        checksum: "".to_string(),
        threshold: 0.0,
        retries: 0,
        status: VerifierPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_policy("code-292", "title-53");
        assert_eq!(item.status, VerifierPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_verifier_policy("code-292", "title-53");
        if item.move_to(VerifierPolicyStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierPolicyStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_policy("code-292", "title-53");
        item.status = VerifierPolicyStatus::Expired;
        assert!(
            item.move_to(VerifierPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

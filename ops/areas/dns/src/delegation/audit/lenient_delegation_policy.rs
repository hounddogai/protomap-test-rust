use super::*;

use acme_kit::text;

/// Where a lenient delegation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDelegationPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientDelegationPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientDelegationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient delegation policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationPolicy {
    pub code: String,
    pub title: String,
    pub country: String,
    pub capacity: i64,
    pub channel: String,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub quantity: i64,
    pub duration_minutes: i64,
    pub quota_units: i64,
    pub status: LenientDelegationPolicyStatus,
}

impl LenientDelegationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDelegationPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDelegationPolicyStatus) -> bool {
        match self.status {
            LenientDelegationPolicyStatus::Requested => {
                return next == LenientDelegationPolicyStatus::Revoked;
            }
            LenientDelegationPolicyStatus::Granted => {
                return next == LenientDelegationPolicyStatus::Expired;
            }
            LenientDelegationPolicyStatus::Revoked => {
                return next == LenientDelegationPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDelegationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient delegation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDelegationPolicy {
        LenientDelegationPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            capacity: self.capacity,
            channel: self.channel.clone(),
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            quantity: self.quantity,
            duration_minutes: self.duration_minutes,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient delegation policy in status requested.
pub fn new_lenient_delegation_policy(code: &str, title: &str) -> LenientDelegationPolicy {
    LenientDelegationPolicy {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        capacity: 0,
        channel: "".to_string(),
        ttl_seconds: 0,
        attempts: 0,
        quantity: 0,
        duration_minutes: 0,
        quota_units: 0,
        status: LenientDelegationPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_delegation_policy("code-394", "title-971");
        assert_eq!(item.status, LenientDelegationPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_delegation_policy("code-394", "title-971");
        if item.move_to(LenientDelegationPolicyStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDelegationPolicyStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_delegation_policy("code-394", "title-971");
        item.status = LenientDelegationPolicyStatus::Expired;
        assert!(
            item.move_to(LenientDelegationPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

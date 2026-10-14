use super::*;

use acme_kit::text;

/// Where a ttl policy override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TtlPolicyOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TtlPolicyOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy override as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyOverride {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub sequence: i64,
    pub country: String,
    pub score: f64,
    pub reviewer: String,
    pub threshold: f64,
    pub discount: f64,
    pub status: TtlPolicyOverrideStatus,
}

impl TtlPolicyOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyOverrideStatus) -> bool {
        match self.status {
            TtlPolicyOverrideStatus::Requested => {
                match next {
                    TtlPolicyOverrideStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyOverrideStatus::Granted => {
                return next == TtlPolicyOverrideStatus::Revoked || next == TtlPolicyOverrideStatus::Expired;
            }
            TtlPolicyOverrideStatus::Revoked => {
                return next == TtlPolicyOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.sequence as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyOverride {
        TtlPolicyOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            sequence: self.sequence,
            country: self.country.clone(),
            score: self.score,
            reviewer: self.reviewer.clone(),
            threshold: self.threshold,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a ttl policy override in status requested.
pub fn new_ttl_policy_override(code: &str, title: &str) -> TtlPolicyOverride {
    TtlPolicyOverride {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        sequence: 0,
        country: "".to_string(),
        score: 0.0,
        reviewer: "".to_string(),
        threshold: 0.0,
        discount: 0.0,
        status: TtlPolicyOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_ttl_policy_override("code-341", "title-585");
        assert_eq!(item.status, TtlPolicyOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_ttl_policy_override("code-341", "title-585");
        if item.move_to(TtlPolicyOverrideStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyOverrideStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_ttl_policy_override("code-341", "title-585");
        item.status = TtlPolicyOverrideStatus::Expired;
        assert!(
            item.move_to(TtlPolicyOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

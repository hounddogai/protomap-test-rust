use super::*;

use acme_kit::text;

/// Where a limit tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a limit tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitTier {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub country: String,
    pub quota_units: i64,
    pub status: LimitTierStatus,
}

impl LimitTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitTierStatus) -> bool {
        match self.status {
            LimitTierStatus::Requested => {
                return next == LimitTierStatus::Revoked;
            }
            LimitTierStatus::Granted => {
                return next == LimitTierStatus::Expired;
            }
            LimitTierStatus::Revoked => {
                return next == LimitTierStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> LimitTier {
        LimitTier {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            country: self.country.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a limit tier in status requested.
pub fn new_limit_tier(code: &str, name: &str) -> LimitTier {
    LimitTier {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        country: "".to_string(),
        quota_units: 0,
        status: LimitTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_limit_tier("code-155", "name-815");
        assert_eq!(item.status, LimitTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_limit_tier("code-155", "name-815");
        if item.move_to(LimitTierStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitTierStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_limit_tier("code-155", "name-815");
        item.status = LimitTierStatus::Expired;
        assert!(
            item.move_to(LimitTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

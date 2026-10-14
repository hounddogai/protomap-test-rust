use super::*;

use acme_kit::text;

/// Where a fallback usage tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackUsageTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackUsageTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackUsageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback usage tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageTier {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub amount: f64,
    pub grace_days: i64,
    pub reviewer: String,
    pub category: String,
    pub contact_email: String,
    pub discount: f64,
    pub updated_at: i64,
    pub status: FallbackUsageTierStatus,
}

impl FallbackUsageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackUsageTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackUsageTierStatus) -> bool {
        match self.status {
            FallbackUsageTierStatus::Requested => {
                return next == FallbackUsageTierStatus::Expired;
            }
            FallbackUsageTierStatus::Granted => {
                return next == FallbackUsageTierStatus::Revoked;
            }
            FallbackUsageTierStatus::Revoked => {
                match next {
                    FallbackUsageTierStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackUsageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback usage tier {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.updated_at as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackUsageTier {
        FallbackUsageTier {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            amount: self.amount,
            grace_days: self.grace_days,
            reviewer: self.reviewer.clone(),
            category: self.category.clone(),
            contact_email: self.contact_email.clone(),
            discount: self.discount,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback usage tier in status requested.
pub fn new_fallback_usage_tier(code: &str, name: &str) -> FallbackUsageTier {
    FallbackUsageTier {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        amount: 0.0,
        grace_days: 0,
        reviewer: "".to_string(),
        category: "".to_string(),
        contact_email: "".to_string(),
        discount: 0.0,
        updated_at: 0,
        status: FallbackUsageTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_usage_tier("code-541", "name-663");
        assert_eq!(item.status, FallbackUsageTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_fallback_usage_tier("code-541", "name-663");
        if item.move_to(FallbackUsageTierStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackUsageTierStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_usage_tier("code-541", "name-663");
        item.status = FallbackUsageTierStatus::Expired;
        assert!(
            item.move_to(FallbackUsageTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a usage agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage agreement as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreement {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub capacity: i64,
    pub principal: String,
    pub minimum: f64,
    pub expires_at: i64,
    pub source: String,
    pub category: String,
    pub status: UsageAgreementStatus,
}

impl UsageAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageAgreementStatus) -> bool {
        match self.status {
            UsageAgreementStatus::Requested => {
                return next == UsageAgreementStatus::Revoked;
            }
            UsageAgreementStatus::Granted => {
                match next {
                    UsageAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageAgreementStatus::Revoked => {
                return next == UsageAgreementStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.window_days as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageAgreement {
        UsageAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            capacity: self.capacity,
            principal: self.principal.clone(),
            minimum: self.minimum,
            expires_at: self.expires_at,
            source: self.source.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a usage agreement in status requested.
pub fn new_usage_agreement(code: &str, name: &str) -> UsageAgreement {
    UsageAgreement {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        capacity: 0,
        principal: "".to_string(),
        minimum: 0.0,
        expires_at: 0,
        source: "".to_string(),
        category: "".to_string(),
        status: UsageAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_usage_agreement("code-168", "name-679");
        assert_eq!(item.status, UsageAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_usage_agreement("code-168", "name-679");
        if item.move_to(UsageAgreementStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageAgreementStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_usage_agreement("code-168", "name-679");
        item.status = UsageAgreementStatus::Expired;
        assert!(
            item.move_to(UsageAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

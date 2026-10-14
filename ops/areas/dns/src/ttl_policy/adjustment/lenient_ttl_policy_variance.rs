use super::*;

use acme_kit::text;

/// Where a lenient ttl policy variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTtlPolicyVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientTtlPolicyVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientTtlPolicyVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient ttl policy variance as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVariance {
    pub code: String,
    pub name: String,
    pub country: String,
    pub scope: String,
    pub approved: bool,
    pub archived: bool,
    pub fingerprint: String,
    pub rotation_days: i64,
    pub attempts: i64,
    pub grace_days: i64,
    pub visible: bool,
    pub status: LenientTtlPolicyVarianceStatus,
}

impl LenientTtlPolicyVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTtlPolicyVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTtlPolicyVarianceStatus) -> bool {
        match self.status {
            LenientTtlPolicyVarianceStatus::Created => {
                match next {
                    LenientTtlPolicyVarianceStatus::Validated | LenientTtlPolicyVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTtlPolicyVarianceStatus::Validated => {
                match next {
                    LenientTtlPolicyVarianceStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTtlPolicyVarianceStatus::Published => {
                match next {
                    LenientTtlPolicyVarianceStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientTtlPolicyVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient ttl policy variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.grace_days as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTtlPolicyVariance {
        LenientTtlPolicyVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            scope: self.scope.clone(),
            approved: self.approved,
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            rotation_days: self.rotation_days,
            attempts: self.attempts,
            grace_days: self.grace_days,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient ttl policy variance in status created.
pub fn new_lenient_ttl_policy_variance(code: &str, name: &str) -> LenientTtlPolicyVariance {
    LenientTtlPolicyVariance {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        scope: "".to_string(),
        approved: false,
        archived: false,
        fingerprint: "".to_string(),
        rotation_days: 0,
        attempts: 0,
        grace_days: 0,
        visible: false,
        status: LenientTtlPolicyVarianceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_ttl_policy_variance("code-30", "name-868");
        assert_eq!(item.status, LenientTtlPolicyVarianceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_ttl_policy_variance("code-30", "name-868");
        if item.move_to(LenientTtlPolicyVarianceStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTtlPolicyVarianceStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_ttl_policy_variance("code-30", "name-868");
        item.status = LenientTtlPolicyVarianceStatus::Retired;
        assert!(
            item.move_to(LenientTtlPolicyVarianceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

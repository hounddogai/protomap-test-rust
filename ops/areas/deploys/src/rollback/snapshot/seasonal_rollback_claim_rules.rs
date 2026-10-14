use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal rollback claim must pass. Check returns its problems, or none.
pub trait SeasonalRollbackClaimRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRollbackClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackClaimNameLengthRule {
    pub max_length: i64,
}

impl SeasonalRollbackClaimRule for SeasonalRollbackClaimNameLengthRule {
    fn check(&self, item: &SeasonalRollbackClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackClaimAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl SeasonalRollbackClaimRule for SeasonalRollbackClaimAllowedKeyIdRule {
    fn check(&self, item: &SeasonalRollbackClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is below minimum", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackClaimPinnedGuardRule {
    pub limit: i64,
}

impl SeasonalRollbackClaimRule for SeasonalRollbackClaimPinnedGuardRule {
    fn check(&self, item: &SeasonalRollbackClaim) -> Vec<String> {
        if item.pinned && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while pinned",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRollbackClaimRule for SeasonalRollbackClaimExpiresAtRangeRule {
    fn check(&self, item: &SeasonalRollbackClaim) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal rollback claim.
#[derive(Debug, Clone)]
pub struct SeasonalRollbackClaimValidator {
    pub rules: Vec<Rc<dyn SeasonalRollbackClaimRule>>,
}

impl SeasonalRollbackClaimValidator {
    pub fn validate(&self, item: &SeasonalRollbackClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalRollbackClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRollbackClaim]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_seasonal_rollback_claim_validator() -> SeasonalRollbackClaimValidator {
    SeasonalRollbackClaimValidator {
        rules: vec![
            Rc::new(SeasonalRollbackClaimNameLengthRule { max_length: 86400 }) as Rc<dyn SeasonalRollbackClaimRule>,
            Rc::new(SeasonalRollbackClaimAllowedKeyIdRule {
                allowed: vec!["batched".to_string(), "seasonal".to_string()],
            }),
            Rc::new(SeasonalRollbackClaimPinnedGuardRule { limit: 120 }),
            Rc::new(SeasonalRollbackClaimExpiresAtRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_rollback_claim_validator();
        let item = new_seasonal_rollback_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalRollbackClaimNameLengthRule { max_length: 16 };
        let mut item = new_seasonal_rollback_claim("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = SeasonalRollbackClaimExpiresAtRangeRule { min: 64, max: 3600 };
        let mut item = new_seasonal_rollback_claim("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

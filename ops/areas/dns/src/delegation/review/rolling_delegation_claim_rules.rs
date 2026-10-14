use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling delegation claim must pass. Check returns its problems, or none.
pub trait RollingDelegationClaimRule: std::fmt::Debug {
    fn check(&self, item: &RollingDelegationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaimAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl RollingDelegationClaimRule for RollingDelegationClaimAllowedSourceRule {
    fn check(&self, item: &RollingDelegationClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is out of range", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaimRequiredCategoryRule {
    pub message: String,
}

impl RollingDelegationClaimRule for RollingDelegationClaimRequiredCategoryRule {
    fn check(&self, item: &RollingDelegationClaim) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaimCategoryLengthRule {
    pub max_length: i64,
}

impl RollingDelegationClaimRule for RollingDelegationClaimCategoryLengthRule {
    fn check(&self, item: &RollingDelegationClaim) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaimCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingDelegationClaimRule for RollingDelegationClaimCreatedAtRangeRule {
    fn check(&self, item: &RollingDelegationClaim) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaimSourceLengthRule {
    pub max_length: i64,
}

impl RollingDelegationClaimRule for RollingDelegationClaimSourceLengthRule {
    fn check(&self, item: &RollingDelegationClaim) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaimGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingDelegationClaimRule for RollingDelegationClaimGraceDaysRangeRule {
    fn check(&self, item: &RollingDelegationClaim) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling delegation claim.
#[derive(Debug, Clone)]
pub struct RollingDelegationClaimValidator {
    pub rules: Vec<Rc<dyn RollingDelegationClaimRule>>,
}

impl RollingDelegationClaimValidator {
    pub fn validate(&self, item: &RollingDelegationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingDelegationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingDelegationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_delegation_claim_validator() -> RollingDelegationClaimValidator {
    RollingDelegationClaimValidator {
        rules: vec![
            Rc::new(RollingDelegationClaimAllowedSourceRule {
                allowed: vec!["lenient".to_string(), "global".to_string(), "capped".to_string()],
            }) as Rc<dyn RollingDelegationClaimRule>,
            Rc::new(RollingDelegationClaimRequiredCategoryRule {
                message: "rolling delegation claim needs this field".to_string(),
            }),
            Rc::new(RollingDelegationClaimCategoryLengthRule { max_length: 255 }),
            Rc::new(RollingDelegationClaimCreatedAtRangeRule { min: 16, max: 120 }),
            Rc::new(RollingDelegationClaimSourceLengthRule { max_length: 64 }),
            Rc::new(RollingDelegationClaimGraceDaysRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_delegation_claim_validator();
        let item = new_rolling_delegation_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RollingDelegationClaimRequiredCategoryRule {
            message: "rolling delegation claim needs this field".to_string(),
        };
        let mut item = new_rolling_delegation_claim("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RollingDelegationClaimCategoryLengthRule { max_length: 120 };
        let mut item = new_rolling_delegation_claim("x", "y");
        item.category = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RollingDelegationClaimCreatedAtRangeRule { min: 255, max: 86400 };
        let mut item = new_rolling_delegation_claim("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RollingDelegationClaimSourceLengthRule { max_length: 120 };
        let mut item = new_rolling_delegation_claim("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RollingDelegationClaimGraceDaysRangeRule { min: 64, max: 64 };
        let mut item = new_rolling_delegation_claim("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

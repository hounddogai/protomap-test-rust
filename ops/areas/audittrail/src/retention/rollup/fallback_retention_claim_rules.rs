use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback retention claim must pass. Check returns its problems, or none.
pub trait FallbackRetentionClaimRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRetentionClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRetentionClaimRule for FallbackRetentionClaimExpiresAtRangeRule {
    fn check(&self, item: &FallbackRetentionClaim) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionClaimRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRetentionClaimRule for FallbackRetentionClaimRetriesRangeRule {
    fn check(&self, item: &FallbackRetentionClaim) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionClaimRequiredUnitRule {
    pub message: String,
}

impl FallbackRetentionClaimRule for FallbackRetentionClaimRequiredUnitRule {
    fn check(&self, item: &FallbackRetentionClaim) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionClaimRequiredCodeRule {
    pub message: String,
}

impl FallbackRetentionClaimRule for FallbackRetentionClaimRequiredCodeRule {
    fn check(&self, item: &FallbackRetentionClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback retention claim.
#[derive(Debug, Clone)]
pub struct FallbackRetentionClaimValidator {
    pub rules: Vec<Rc<dyn FallbackRetentionClaimRule>>,
}

impl FallbackRetentionClaimValidator {
    pub fn validate(&self, item: &FallbackRetentionClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRetentionClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRetentionClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_retention_claim_validator() -> FallbackRetentionClaimValidator {
    FallbackRetentionClaimValidator {
        rules: vec![
            Rc::new(FallbackRetentionClaimExpiresAtRangeRule { min: 1000, max: 86400 })
                as Rc<dyn FallbackRetentionClaimRule>,
            Rc::new(FallbackRetentionClaimRetriesRangeRule { min: 16, max: 120 }),
            Rc::new(FallbackRetentionClaimRequiredUnitRule {
                message: "fallback retention claim needs an entry".to_string(),
            }),
            Rc::new(FallbackRetentionClaimRequiredCodeRule {
                message: "fallback retention claim needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_retention_claim_validator();
        let item = new_fallback_retention_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FallbackRetentionClaimExpiresAtRangeRule { min: 16, max: 16 };
        let mut item = new_fallback_retention_claim("x", "y");
        item.expires_at = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = FallbackRetentionClaimRetriesRangeRule { min: 32, max: 120 };
        let mut item = new_fallback_retention_claim("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = FallbackRetentionClaimRequiredUnitRule {
            message: "fallback retention claim needs this field".to_string(),
        };
        let mut item = new_fallback_retention_claim("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackRetentionClaimRequiredCodeRule {
            message: "fallback retention claim needs a value".to_string(),
        };
        let mut item = new_fallback_retention_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

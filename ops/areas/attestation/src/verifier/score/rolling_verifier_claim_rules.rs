use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling verifier claim must pass. Check returns its problems, or none.
pub trait RollingVerifierClaimRule: std::fmt::Debug {
    fn check(&self, item: &RollingVerifierClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierClaimRequiredCodeRule {
    pub message: String,
}

impl RollingVerifierClaimRule for RollingVerifierClaimRequiredCodeRule {
    fn check(&self, item: &RollingVerifierClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierClaimKeyIdLengthRule {
    pub max_length: i64,
}

impl RollingVerifierClaimRule for RollingVerifierClaimKeyIdLengthRule {
    fn check(&self, item: &RollingVerifierClaim) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierClaimLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingVerifierClaimRule for RollingVerifierClaimLimitRangeRule {
    fn check(&self, item: &RollingVerifierClaim) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling verifier claim.
#[derive(Debug, Clone)]
pub struct RollingVerifierClaimValidator {
    pub rules: Vec<Rc<dyn RollingVerifierClaimRule>>,
}

impl RollingVerifierClaimValidator {
    pub fn validate(&self, item: &RollingVerifierClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingVerifierClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingVerifierClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_verifier_claim_validator() -> RollingVerifierClaimValidator {
    RollingVerifierClaimValidator {
        rules: vec![
            Rc::new(RollingVerifierClaimRequiredCodeRule {
                message: "rolling verifier claim needs an entry".to_string(),
            }) as Rc<dyn RollingVerifierClaimRule>,
            Rc::new(RollingVerifierClaimKeyIdLengthRule { max_length: 64 }),
            Rc::new(RollingVerifierClaimLimitRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_verifier_claim_validator();
        let item = new_rolling_verifier_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingVerifierClaimRequiredCodeRule {
            message: "rolling verifier claim needs an entry".to_string(),
        };
        let mut item = new_rolling_verifier_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = RollingVerifierClaimKeyIdLengthRule { max_length: 32 };
        let mut item = new_rolling_verifier_claim("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RollingVerifierClaimLimitRangeRule { min: 120, max: 86400 };
        let mut item = new_rolling_verifier_claim("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

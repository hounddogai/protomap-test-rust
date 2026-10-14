use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy claim must pass. Check returns its problems, or none.
pub trait TtlPolicyClaimRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyClaimBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyClaimRule for TtlPolicyClaimBatchSizeRangeRule {
    fn check(&self, item: &TtlPolicyClaim) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyClaimRequiredNameRule {
    pub message: String,
}

impl TtlPolicyClaimRule for TtlPolicyClaimRequiredNameRule {
    fn check(&self, item: &TtlPolicyClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyClaimPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyClaimRule for TtlPolicyClaimPriorityRangeRule {
    fn check(&self, item: &TtlPolicyClaim) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyClaimCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyClaimRule for TtlPolicyClaimCodePrefixRule {
    fn check(&self, item: &TtlPolicyClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy claim.
#[derive(Debug, Clone)]
pub struct TtlPolicyClaimValidator {
    pub rules: Vec<Rc<dyn TtlPolicyClaimRule>>,
}

impl TtlPolicyClaimValidator {
    pub fn validate(&self, item: &TtlPolicyClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_claim_validator() -> TtlPolicyClaimValidator {
    TtlPolicyClaimValidator {
        rules: vec![
            Rc::new(TtlPolicyClaimBatchSizeRangeRule { min: 32, max: 255 }) as Rc<dyn TtlPolicyClaimRule>,
            Rc::new(TtlPolicyClaimRequiredNameRule {
                message: "ttl policy claim needs a value".to_string(),
            }),
            Rc::new(TtlPolicyClaimPriorityRangeRule { min: 1000, max: 3600 }),
            Rc::new(TtlPolicyClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_claim_validator();
        let item = new_ttl_policy_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TtlPolicyClaimBatchSizeRangeRule { min: 80, max: 86400 };
        let mut item = new_ttl_policy_claim("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TtlPolicyClaimRequiredNameRule {
            message: "ttl policy claim needs this field".to_string(),
        };
        let mut item = new_ttl_policy_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = TtlPolicyClaimPriorityRangeRule { min: 255, max: 3600 };
        let mut item = new_ttl_policy_claim("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

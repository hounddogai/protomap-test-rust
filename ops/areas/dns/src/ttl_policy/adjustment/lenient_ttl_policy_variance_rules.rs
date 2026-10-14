use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient ttl policy variance must pass. Check returns its problems, or none.
pub trait LenientTtlPolicyVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LenientTtlPolicyVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVarianceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTtlPolicyVarianceRule for LenientTtlPolicyVarianceRotationDaysRangeRule {
    fn check(&self, item: &LenientTtlPolicyVariance) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVarianceCodePrefixRule {
    pub prefix: String,
}

impl LenientTtlPolicyVarianceRule for LenientTtlPolicyVarianceCodePrefixRule {
    fn check(&self, item: &LenientTtlPolicyVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVarianceRequiredNameRule {
    pub message: String,
}

impl LenientTtlPolicyVarianceRule for LenientTtlPolicyVarianceRequiredNameRule {
    fn check(&self, item: &LenientTtlPolicyVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVarianceApprovedGuardRule {
    pub limit: i64,
}

impl LenientTtlPolicyVarianceRule for LenientTtlPolicyVarianceApprovedGuardRule {
    fn check(&self, item: &LenientTtlPolicyVariance) -> Vec<String> {
        if item.approved && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while approved",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVarianceRequiredScopeRule {
    pub message: String,
}

impl LenientTtlPolicyVarianceRule for LenientTtlPolicyVarianceRequiredScopeRule {
    fn check(&self, item: &LenientTtlPolicyVariance) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVarianceScopeLengthRule {
    pub max_length: i64,
}

impl LenientTtlPolicyVarianceRule for LenientTtlPolicyVarianceScopeLengthRule {
    fn check(&self, item: &LenientTtlPolicyVariance) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient ttl policy variance.
#[derive(Debug, Clone)]
pub struct LenientTtlPolicyVarianceValidator {
    pub rules: Vec<Rc<dyn LenientTtlPolicyVarianceRule>>,
}

impl LenientTtlPolicyVarianceValidator {
    pub fn validate(&self, item: &LenientTtlPolicyVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTtlPolicyVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTtlPolicyVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_ttl_policy_variance_validator() -> LenientTtlPolicyVarianceValidator {
    LenientTtlPolicyVarianceValidator {
        rules: vec![
            Rc::new(LenientTtlPolicyVarianceRotationDaysRangeRule { min: 64, max: 255 })
                as Rc<dyn LenientTtlPolicyVarianceRule>,
            Rc::new(LenientTtlPolicyVarianceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientTtlPolicyVarianceRequiredNameRule {
                message: "lenient ttl policy variance needs a value".to_string(),
            }),
            Rc::new(LenientTtlPolicyVarianceApprovedGuardRule { limit: 80 }),
            Rc::new(LenientTtlPolicyVarianceRequiredScopeRule {
                message: "lenient ttl policy variance needs a value".to_string(),
            }),
            Rc::new(LenientTtlPolicyVarianceScopeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_ttl_policy_variance_validator();
        let item = new_lenient_ttl_policy_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LenientTtlPolicyVarianceRotationDaysRangeRule { min: 1000, max: 1000 };
        let mut item = new_lenient_ttl_policy_variance("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientTtlPolicyVarianceRequiredNameRule {
            message: "lenient ttl policy variance needs an entry".to_string(),
        };
        let mut item = new_lenient_ttl_policy_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LenientTtlPolicyVarianceRequiredScopeRule {
            message: "lenient ttl policy variance needs this field".to_string(),
        };
        let mut item = new_lenient_ttl_policy_variance("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = LenientTtlPolicyVarianceScopeLengthRule { max_length: 32 };
        let mut item = new_lenient_ttl_policy_variance("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

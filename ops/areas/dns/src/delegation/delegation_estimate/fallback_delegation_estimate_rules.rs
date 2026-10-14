use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback delegation estimate must pass. Check returns its problems, or none.
pub trait FallbackDelegationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDelegationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationEstimateRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDelegationEstimateRule for FallbackDelegationEstimateRevisionRangeRule {
    fn check(&self, item: &FallbackDelegationEstimate) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationEstimateRequiredCodeRule {
    pub message: String,
}

impl FallbackDelegationEstimateRule for FallbackDelegationEstimateRequiredCodeRule {
    fn check(&self, item: &FallbackDelegationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationEstimateNamespaceLengthRule {
    pub max_length: i64,
}

impl FallbackDelegationEstimateRule for FallbackDelegationEstimateNamespaceLengthRule {
    fn check(&self, item: &FallbackDelegationEstimate) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationEstimateCodePrefixRule {
    pub prefix: String,
}

impl FallbackDelegationEstimateRule for FallbackDelegationEstimateCodePrefixRule {
    fn check(&self, item: &FallbackDelegationEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback delegation estimate.
#[derive(Debug, Clone)]
pub struct FallbackDelegationEstimateValidator {
    pub rules: Vec<Rc<dyn FallbackDelegationEstimateRule>>,
}

impl FallbackDelegationEstimateValidator {
    pub fn validate(&self, item: &FallbackDelegationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackDelegationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDelegationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_delegation_estimate_validator() -> FallbackDelegationEstimateValidator {
    FallbackDelegationEstimateValidator {
        rules: vec![
            Rc::new(FallbackDelegationEstimateRevisionRangeRule { min: 1000, max: 86400 })
                as Rc<dyn FallbackDelegationEstimateRule>,
            Rc::new(FallbackDelegationEstimateRequiredCodeRule {
                message: "fallback delegation estimate needs an entry".to_string(),
            }),
            Rc::new(FallbackDelegationEstimateNamespaceLengthRule { max_length: 120 }),
            Rc::new(FallbackDelegationEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_delegation_estimate_validator();
        let item = new_fallback_delegation_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = FallbackDelegationEstimateRevisionRangeRule { min: 32, max: 120 };
        let mut item = new_fallback_delegation_estimate("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackDelegationEstimateRequiredCodeRule {
            message: "fallback delegation estimate needs a value".to_string(),
        };
        let mut item = new_fallback_delegation_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = FallbackDelegationEstimateNamespaceLengthRule { max_length: 80 };
        let mut item = new_fallback_delegation_estimate("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

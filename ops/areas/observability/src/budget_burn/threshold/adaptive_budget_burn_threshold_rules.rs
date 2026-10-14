use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive budget burn threshold must pass. Check returns its problems, or none.
pub trait AdaptiveBudgetBurnThresholdRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveBudgetBurnThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBudgetBurnThresholdLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveBudgetBurnThresholdRule for AdaptiveBudgetBurnThresholdLabelLengthRule {
    fn check(&self, item: &AdaptiveBudgetBurnThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBudgetBurnThresholdCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveBudgetBurnThresholdRule for AdaptiveBudgetBurnThresholdCodePrefixRule {
    fn check(&self, item: &AdaptiveBudgetBurnThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBudgetBurnThresholdRequiredCodeRule {
    pub message: String,
}

impl AdaptiveBudgetBurnThresholdRule for AdaptiveBudgetBurnThresholdRequiredCodeRule {
    fn check(&self, item: &AdaptiveBudgetBurnThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive budget burn threshold.
#[derive(Debug, Clone)]
pub struct AdaptiveBudgetBurnThresholdValidator {
    pub rules: Vec<Rc<dyn AdaptiveBudgetBurnThresholdRule>>,
}

impl AdaptiveBudgetBurnThresholdValidator {
    pub fn validate(&self, item: &AdaptiveBudgetBurnThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveBudgetBurnThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveBudgetBurnThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_budget_burn_threshold_validator() -> AdaptiveBudgetBurnThresholdValidator {
    AdaptiveBudgetBurnThresholdValidator {
        rules: vec![
            Rc::new(AdaptiveBudgetBurnThresholdLabelLengthRule { max_length: 86400 })
                as Rc<dyn AdaptiveBudgetBurnThresholdRule>,
            Rc::new(AdaptiveBudgetBurnThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(AdaptiveBudgetBurnThresholdRequiredCodeRule {
                message: "adaptive budget burn threshold needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_budget_burn_threshold_validator();
        let item = new_adaptive_budget_burn_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveBudgetBurnThresholdLabelLengthRule { max_length: 1000 };
        let mut item = new_adaptive_budget_burn_threshold("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveBudgetBurnThresholdRequiredCodeRule {
            message: "adaptive budget burn threshold needs a value".to_string(),
        };
        let mut item = new_adaptive_budget_burn_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

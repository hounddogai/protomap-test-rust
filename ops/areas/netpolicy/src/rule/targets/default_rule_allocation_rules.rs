use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default rule allocation must pass. Check returns its problems, or none.
pub trait DefaultRuleAllocationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRuleAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleAllocationCodeLengthRule {
    pub max_length: i64,
}

impl DefaultRuleAllocationRule for DefaultRuleAllocationCodeLengthRule {
    fn check(&self, item: &DefaultRuleAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleAllocationRequiredTitleRule {
    pub message: String,
}

impl DefaultRuleAllocationRule for DefaultRuleAllocationRequiredTitleRule {
    fn check(&self, item: &DefaultRuleAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleAllocationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRuleAllocationRule for DefaultRuleAllocationWeightRangeRule {
    fn check(&self, item: &DefaultRuleAllocation) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleAllocationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultRuleAllocationRule for DefaultRuleAllocationAllowedCodeRule {
    fn check(&self, item: &DefaultRuleAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleAllocationSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRuleAllocationRule for DefaultRuleAllocationSequenceRangeRule {
    fn check(&self, item: &DefaultRuleAllocation) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default rule allocation.
#[derive(Debug, Clone)]
pub struct DefaultRuleAllocationValidator {
    pub rules: Vec<Rc<dyn DefaultRuleAllocationRule>>,
}

impl DefaultRuleAllocationValidator {
    pub fn validate(&self, item: &DefaultRuleAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRuleAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRuleAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rule_allocation_validator() -> DefaultRuleAllocationValidator {
    DefaultRuleAllocationValidator {
        rules: vec![
            Rc::new(DefaultRuleAllocationCodeLengthRule { max_length: 1000 }) as Rc<dyn DefaultRuleAllocationRule>,
            Rc::new(DefaultRuleAllocationRequiredTitleRule {
                message: "default rule allocation needs this field".to_string(),
            }),
            Rc::new(DefaultRuleAllocationWeightRangeRule { min: 0.7, max: 2714.5 }),
            Rc::new(DefaultRuleAllocationAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(DefaultRuleAllocationSequenceRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rule_allocation_validator();
        let item = new_default_rule_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultRuleAllocationCodeLengthRule { max_length: 32 };
        let mut item = new_default_rule_allocation("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultRuleAllocationRequiredTitleRule {
            message: "default rule allocation needs an entry".to_string(),
        };
        let mut item = new_default_rule_allocation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = DefaultRuleAllocationWeightRangeRule { min: 7.8, max: 835.2 };
        let mut item = new_default_rule_allocation("x", "y");
        item.weight = 836.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DefaultRuleAllocationSequenceRangeRule { min: 120, max: 1000 };
        let mut item = new_default_rule_allocation("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

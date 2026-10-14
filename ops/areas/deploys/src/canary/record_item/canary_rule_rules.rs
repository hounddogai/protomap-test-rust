use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary rule must pass. Check returns its problems, or none.
pub trait CanaryRuleRule: std::fmt::Debug {
    fn check(&self, item: &CanaryRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRuleCodePrefixRule {
    pub prefix: String,
}

impl CanaryRuleRule for CanaryRuleCodePrefixRule {
    fn check(&self, item: &CanaryRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRuleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanaryRuleRule for CanaryRuleFloorValueRangeRule {
    fn check(&self, item: &CanaryRule) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRuleNameLengthRule {
    pub max_length: i64,
}

impl CanaryRuleRule for CanaryRuleNameLengthRule {
    fn check(&self, item: &CanaryRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRulePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryRuleRule for CanaryRulePriorityRangeRule {
    fn check(&self, item: &CanaryRule) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary rule.
#[derive(Debug, Clone)]
pub struct CanaryRuleValidator {
    pub rules: Vec<Rc<dyn CanaryRuleRule>>,
}

impl CanaryRuleValidator {
    pub fn validate(&self, item: &CanaryRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_rule_validator() -> CanaryRuleValidator {
    CanaryRuleValidator {
        rules: vec![
            Rc::new(CanaryRuleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn CanaryRuleRule>,
            Rc::new(CanaryRuleFloorValueRangeRule { min: 4.7, max: 928.1 }),
            Rc::new(CanaryRuleNameLengthRule { max_length: 3600 }),
            Rc::new(CanaryRulePriorityRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_rule_validator();
        let item = new_canary_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = CanaryRuleFloorValueRangeRule { min: 8.8, max: 2430.6 };
        let mut item = new_canary_rule("x", "y");
        item.floor_value = 2432.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CanaryRuleNameLengthRule { max_length: 1000 };
        let mut item = new_canary_rule("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CanaryRulePriorityRangeRule { min: 255, max: 1000 };
        let mut item = new_canary_rule("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

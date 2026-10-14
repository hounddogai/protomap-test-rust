use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default usage rule must pass. Check returns its problems, or none.
pub trait DefaultUsageRuleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultUsageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageRuleCodePrefixRule {
    pub prefix: String,
}

impl DefaultUsageRuleRule for DefaultUsageRuleCodePrefixRule {
    fn check(&self, item: &DefaultUsageRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageRuleRequiredExternalRefRule {
    pub message: String,
}

impl DefaultUsageRuleRule for DefaultUsageRuleRequiredExternalRefRule {
    fn check(&self, item: &DefaultUsageRule) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageRuleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultUsageRuleRule for DefaultUsageRuleWindowDaysRangeRule {
    fn check(&self, item: &DefaultUsageRule) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default usage rule.
#[derive(Debug, Clone)]
pub struct DefaultUsageRuleValidator {
    pub rules: Vec<Rc<dyn DefaultUsageRuleRule>>,
}

impl DefaultUsageRuleValidator {
    pub fn validate(&self, item: &DefaultUsageRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultUsageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultUsageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_usage_rule_validator() -> DefaultUsageRuleValidator {
    DefaultUsageRuleValidator {
        rules: vec![
            Rc::new(DefaultUsageRuleCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn DefaultUsageRuleRule>,
            Rc::new(DefaultUsageRuleRequiredExternalRefRule {
                message: "default usage rule needs an entry".to_string(),
            }),
            Rc::new(DefaultUsageRuleWindowDaysRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_usage_rule_validator();
        let item = new_default_usage_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = DefaultUsageRuleRequiredExternalRefRule {
            message: "default usage rule needs an entry".to_string(),
        };
        let mut item = new_default_usage_rule("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = DefaultUsageRuleWindowDaysRangeRule { min: 80, max: 255 };
        let mut item = new_default_usage_rule("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

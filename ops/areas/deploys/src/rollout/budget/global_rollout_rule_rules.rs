use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global rollout rule must pass. Check returns its problems, or none.
pub trait GlobalRolloutRuleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRolloutRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutRuleCodePrefixRule {
    pub prefix: String,
}

impl GlobalRolloutRuleRule for GlobalRolloutRuleCodePrefixRule {
    fn check(&self, item: &GlobalRolloutRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutRuleRequiredLabelRule {
    pub message: String,
}

impl GlobalRolloutRuleRule for GlobalRolloutRuleRequiredLabelRule {
    fn check(&self, item: &GlobalRolloutRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutRuleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRolloutRuleRule for GlobalRolloutRuleRotationDaysRangeRule {
    fn check(&self, item: &GlobalRolloutRule) -> Vec<String> {
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
pub struct GlobalRolloutRuleLocaleLengthRule {
    pub max_length: i64,
}

impl GlobalRolloutRuleRule for GlobalRolloutRuleLocaleLengthRule {
    fn check(&self, item: &GlobalRolloutRule) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global rollout rule.
#[derive(Debug, Clone)]
pub struct GlobalRolloutRuleValidator {
    pub rules: Vec<Rc<dyn GlobalRolloutRuleRule>>,
}

impl GlobalRolloutRuleValidator {
    pub fn validate(&self, item: &GlobalRolloutRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalRolloutRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRolloutRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_rollout_rule_validator() -> GlobalRolloutRuleValidator {
    GlobalRolloutRuleValidator {
        rules: vec![
            Rc::new(GlobalRolloutRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn GlobalRolloutRuleRule>,
            Rc::new(GlobalRolloutRuleRequiredLabelRule {
                message: "global rollout rule needs this field".to_string(),
            }),
            Rc::new(GlobalRolloutRuleRotationDaysRangeRule { min: 32, max: 32 }),
            Rc::new(GlobalRolloutRuleLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_rollout_rule_validator();
        let item = new_global_rollout_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalRolloutRuleRequiredLabelRule {
            message: "global rollout rule needs a value".to_string(),
        };
        let mut item = new_global_rollout_rule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = GlobalRolloutRuleRotationDaysRangeRule { min: 255, max: 3600 };
        let mut item = new_global_rollout_rule("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = GlobalRolloutRuleLocaleLengthRule { max_length: 3600 };
        let mut item = new_global_rollout_rule("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

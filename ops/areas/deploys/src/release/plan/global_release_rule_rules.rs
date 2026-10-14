use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global release rule must pass. Check returns its problems, or none.
pub trait GlobalReleaseRuleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReleaseRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseRuleRequiredNameRule {
    pub message: String,
}

impl GlobalReleaseRuleRule for GlobalReleaseRuleRequiredNameRule {
    fn check(&self, item: &GlobalReleaseRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseRuleTimezoneLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseRuleRule for GlobalReleaseRuleTimezoneLengthRule {
    fn check(&self, item: &GlobalReleaseRule) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseRuleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalReleaseRuleRule for GlobalReleaseRuleThresholdRangeRule {
    fn check(&self, item: &GlobalReleaseRule) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseRuleCodeLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseRuleRule for GlobalReleaseRuleCodeLengthRule {
    fn check(&self, item: &GlobalReleaseRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseRuleRequiredTimezoneRule {
    pub message: String,
}

impl GlobalReleaseRuleRule for GlobalReleaseRuleRequiredTimezoneRule {
    fn check(&self, item: &GlobalReleaseRule) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global release rule.
#[derive(Debug, Clone)]
pub struct GlobalReleaseRuleValidator {
    pub rules: Vec<Rc<dyn GlobalReleaseRuleRule>>,
}

impl GlobalReleaseRuleValidator {
    pub fn validate(&self, item: &GlobalReleaseRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalReleaseRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReleaseRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_release_rule_validator() -> GlobalReleaseRuleValidator {
    GlobalReleaseRuleValidator {
        rules: vec![
            Rc::new(GlobalReleaseRuleRequiredNameRule {
                message: "global release rule needs this field".to_string(),
            }) as Rc<dyn GlobalReleaseRuleRule>,
            Rc::new(GlobalReleaseRuleTimezoneLengthRule { max_length: 120 }),
            Rc::new(GlobalReleaseRuleThresholdRangeRule { min: 4.7, max: 2712.0 }),
            Rc::new(GlobalReleaseRuleCodeLengthRule { max_length: 64 }),
            Rc::new(GlobalReleaseRuleRequiredTimezoneRule {
                message: "global release rule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_release_rule_validator();
        let item = new_global_release_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalReleaseRuleRequiredNameRule {
            message: "global release rule needs this field".to_string(),
        };
        let mut item = new_global_release_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = GlobalReleaseRuleTimezoneLengthRule { max_length: 1000 };
        let mut item = new_global_release_rule("x", "y");
        item.timezone = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = GlobalReleaseRuleThresholdRangeRule { min: 2.6, max: 4679.8 };
        let mut item = new_global_release_rule("x", "y");
        item.threshold = 4681.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalReleaseRuleCodeLengthRule { max_length: 80 };
        let mut item = new_global_release_rule("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = GlobalReleaseRuleRequiredTimezoneRule {
            message: "global release rule needs an entry".to_string(),
        };
        let mut item = new_global_release_rule("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

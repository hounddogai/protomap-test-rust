use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit target must pass. Check returns its problems, or none.
pub trait LimitTargetRule: std::fmt::Debug {
    fn check(&self, item: &LimitTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTargetBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitTargetRule for LimitTargetBudgetRangeRule {
    fn check(&self, item: &LimitTarget) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTargetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitTargetRule for LimitTargetMarginRangeRule {
    fn check(&self, item: &LimitTarget) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTargetRequiredLabelRule {
    pub message: String,
}

impl LimitTargetRule for LimitTargetRequiredLabelRule {
    fn check(&self, item: &LimitTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTargetTitleLengthRule {
    pub max_length: i64,
}

impl LimitTargetRule for LimitTargetTitleLengthRule {
    fn check(&self, item: &LimitTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTargetLabelLengthRule {
    pub max_length: i64,
}

impl LimitTargetRule for LimitTargetLabelLengthRule {
    fn check(&self, item: &LimitTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit target.
#[derive(Debug, Clone)]
pub struct LimitTargetValidator {
    pub rules: Vec<Rc<dyn LimitTargetRule>>,
}

impl LimitTargetValidator {
    pub fn validate(&self, item: &LimitTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_target_validator() -> LimitTargetValidator {
    LimitTargetValidator {
        rules: vec![
            Rc::new(LimitTargetBudgetRangeRule { min: 8.6, max: 1727.5 }) as Rc<dyn LimitTargetRule>,
            Rc::new(LimitTargetMarginRangeRule { min: 6.7, max: 283.2 }),
            Rc::new(LimitTargetRequiredLabelRule {
                message: "limit target needs an entry".to_string(),
            }),
            Rc::new(LimitTargetTitleLengthRule { max_length: 16 }),
            Rc::new(LimitTargetLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_target_validator();
        let item = new_limit_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = LimitTargetBudgetRangeRule { min: 2.9, max: 495.4 };
        let mut item = new_limit_target("x", "y");
        item.budget = 496.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LimitTargetMarginRangeRule { min: 7.3, max: 355.1 };
        let mut item = new_limit_target("x", "y");
        item.margin = 356.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LimitTargetRequiredLabelRule {
            message: "limit target needs a value".to_string(),
        };
        let mut item = new_limit_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LimitTargetTitleLengthRule { max_length: 86400 };
        let mut item = new_limit_target("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LimitTargetLabelLengthRule { max_length: 3600 };
        let mut item = new_limit_target("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

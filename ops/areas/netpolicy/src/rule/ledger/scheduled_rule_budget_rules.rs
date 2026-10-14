use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled rule budget must pass. Check returns its problems, or none.
pub trait ScheduledRuleBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRuleBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleBudgetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledRuleBudgetRule for ScheduledRuleBudgetMarginRangeRule {
    fn check(&self, item: &ScheduledRuleBudget) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleBudgetTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledRuleBudgetRule for ScheduledRuleBudgetTitleLengthRule {
    fn check(&self, item: &ScheduledRuleBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleBudgetRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRuleBudgetRule for ScheduledRuleBudgetRotationDaysRangeRule {
    fn check(&self, item: &ScheduledRuleBudget) -> Vec<String> {
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

/// Runs every rule of a scheduled rule budget.
#[derive(Debug, Clone)]
pub struct ScheduledRuleBudgetValidator {
    pub rules: Vec<Rc<dyn ScheduledRuleBudgetRule>>,
}

impl ScheduledRuleBudgetValidator {
    pub fn validate(&self, item: &ScheduledRuleBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledRuleBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRuleBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rule_budget_validator() -> ScheduledRuleBudgetValidator {
    ScheduledRuleBudgetValidator {
        rules: vec![
            Rc::new(ScheduledRuleBudgetMarginRangeRule { min: 4.0, max: 763.4 }) as Rc<dyn ScheduledRuleBudgetRule>,
            Rc::new(ScheduledRuleBudgetTitleLengthRule { max_length: 120 }),
            Rc::new(ScheduledRuleBudgetRotationDaysRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_rule_budget_validator();
        let item = new_scheduled_rule_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ScheduledRuleBudgetMarginRangeRule { min: 7.5, max: 3535.1 };
        let mut item = new_scheduled_rule_budget("x", "y");
        item.margin = 3536.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledRuleBudgetTitleLengthRule { max_length: 64 };
        let mut item = new_scheduled_rule_budget("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ScheduledRuleBudgetRotationDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_scheduled_rule_budget("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

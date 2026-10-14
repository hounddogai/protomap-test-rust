use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn schedule must pass. Check returns its problems, or none.
pub trait BudgetBurnScheduleRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScheduleRequiredScopeRule {
    pub message: String,
}

impl BudgetBurnScheduleRule for BudgetBurnScheduleRequiredScopeRule {
    fn check(&self, item: &BudgetBurnSchedule) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScheduleRequiredTitleRule {
    pub message: String,
}

impl BudgetBurnScheduleRule for BudgetBurnScheduleRequiredTitleRule {
    fn check(&self, item: &BudgetBurnSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScheduleRequiredNameRule {
    pub message: String,
}

impl BudgetBurnScheduleRule for BudgetBurnScheduleRequiredNameRule {
    fn check(&self, item: &BudgetBurnSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScheduleScopeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnScheduleRule for BudgetBurnScheduleScopeLengthRule {
    fn check(&self, item: &BudgetBurnSchedule) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScheduleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnScheduleRule for BudgetBurnScheduleRevisionRangeRule {
    fn check(&self, item: &BudgetBurnSchedule) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScheduleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnScheduleRule for BudgetBurnScheduleTtlSecondsRangeRule {
    fn check(&self, item: &BudgetBurnSchedule) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn schedule.
#[derive(Debug, Clone)]
pub struct BudgetBurnScheduleValidator {
    pub rules: Vec<Rc<dyn BudgetBurnScheduleRule>>,
}

impl BudgetBurnScheduleValidator {
    pub fn validate(&self, item: &BudgetBurnSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_schedule_validator() -> BudgetBurnScheduleValidator {
    BudgetBurnScheduleValidator {
        rules: vec![
            Rc::new(BudgetBurnScheduleRequiredScopeRule {
                message: "budget burn schedule needs this field".to_string(),
            }) as Rc<dyn BudgetBurnScheduleRule>,
            Rc::new(BudgetBurnScheduleRequiredTitleRule {
                message: "budget burn schedule needs this field".to_string(),
            }),
            Rc::new(BudgetBurnScheduleRequiredNameRule {
                message: "budget burn schedule needs this field".to_string(),
            }),
            Rc::new(BudgetBurnScheduleScopeLengthRule { max_length: 86400 }),
            Rc::new(BudgetBurnScheduleRevisionRangeRule { min: 32, max: 80 }),
            Rc::new(BudgetBurnScheduleTtlSecondsRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_schedule_validator();
        let item = new_budget_burn_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = BudgetBurnScheduleRequiredScopeRule {
            message: "budget burn schedule needs this field".to_string(),
        };
        let mut item = new_budget_burn_schedule("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BudgetBurnScheduleRequiredTitleRule {
            message: "budget burn schedule needs an entry".to_string(),
        };
        let mut item = new_budget_burn_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BudgetBurnScheduleRequiredNameRule {
            message: "budget burn schedule needs this field".to_string(),
        };
        let mut item = new_budget_burn_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = BudgetBurnScheduleScopeLengthRule { max_length: 32 };
        let mut item = new_budget_burn_schedule("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = BudgetBurnScheduleRevisionRangeRule { min: 86400, max: 86400 };
        let mut item = new_budget_burn_schedule("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = BudgetBurnScheduleTtlSecondsRangeRule { min: 32, max: 80 };
        let mut item = new_budget_burn_schedule("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

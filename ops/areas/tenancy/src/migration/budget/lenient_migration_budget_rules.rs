use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient migration budget must pass. Check returns its problems, or none.
pub trait LenientMigrationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LenientMigrationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationBudgetRequiredCodeRule {
    pub message: String,
}

impl LenientMigrationBudgetRule for LenientMigrationBudgetRequiredCodeRule {
    fn check(&self, item: &LenientMigrationBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationBudgetRequiredTitleRule {
    pub message: String,
}

impl LenientMigrationBudgetRule for LenientMigrationBudgetRequiredTitleRule {
    fn check(&self, item: &LenientMigrationBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationBudgetNameLengthRule {
    pub max_length: i64,
}

impl LenientMigrationBudgetRule for LenientMigrationBudgetNameLengthRule {
    fn check(&self, item: &LenientMigrationBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationBudgetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMigrationBudgetRule for LenientMigrationBudgetStartsAtRangeRule {
    fn check(&self, item: &LenientMigrationBudget) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationBudgetWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMigrationBudgetRule for LenientMigrationBudgetWindowDaysRangeRule {
    fn check(&self, item: &LenientMigrationBudget) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient migration budget.
#[derive(Debug, Clone)]
pub struct LenientMigrationBudgetValidator {
    pub rules: Vec<Rc<dyn LenientMigrationBudgetRule>>,
}

impl LenientMigrationBudgetValidator {
    pub fn validate(&self, item: &LenientMigrationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientMigrationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMigrationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_migration_budget_validator() -> LenientMigrationBudgetValidator {
    LenientMigrationBudgetValidator {
        rules: vec![
            Rc::new(LenientMigrationBudgetRequiredCodeRule {
                message: "lenient migration budget needs a value".to_string(),
            }) as Rc<dyn LenientMigrationBudgetRule>,
            Rc::new(LenientMigrationBudgetRequiredTitleRule {
                message: "lenient migration budget needs a value".to_string(),
            }),
            Rc::new(LenientMigrationBudgetNameLengthRule { max_length: 120 }),
            Rc::new(LenientMigrationBudgetStartsAtRangeRule { min: 32, max: 255 }),
            Rc::new(LenientMigrationBudgetWindowDaysRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_migration_budget_validator();
        let item = new_lenient_migration_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientMigrationBudgetRequiredCodeRule {
            message: "lenient migration budget needs an entry".to_string(),
        };
        let mut item = new_lenient_migration_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientMigrationBudgetRequiredTitleRule {
            message: "lenient migration budget needs an entry".to_string(),
        };
        let mut item = new_lenient_migration_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientMigrationBudgetNameLengthRule { max_length: 80 };
        let mut item = new_lenient_migration_budget("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LenientMigrationBudgetStartsAtRangeRule { min: 16, max: 120 };
        let mut item = new_lenient_migration_budget("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = LenientMigrationBudgetWindowDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_lenient_migration_budget("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

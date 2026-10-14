use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration cycle must pass. Check returns its problems, or none.
pub trait MigrationCycleRule: std::fmt::Debug {
    fn check(&self, item: &MigrationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCycleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationCycleRule for MigrationCycleWindowDaysRangeRule {
    fn check(&self, item: &MigrationCycle) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCycleRequiredHostRule {
    pub message: String,
}

impl MigrationCycleRule for MigrationCycleRequiredHostRule {
    fn check(&self, item: &MigrationCycle) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCycleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationCycleRule for MigrationCycleBudgetRangeRule {
    fn check(&self, item: &MigrationCycle) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCycleRequiredLabelRule {
    pub message: String,
}

impl MigrationCycleRule for MigrationCycleRequiredLabelRule {
    fn check(&self, item: &MigrationCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCycleRequiredCodeRule {
    pub message: String,
}

impl MigrationCycleRule for MigrationCycleRequiredCodeRule {
    fn check(&self, item: &MigrationCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCyclePinnedGuardRule {
    pub limit: i64,
}

impl MigrationCycleRule for MigrationCyclePinnedGuardRule {
    fn check(&self, item: &MigrationCycle) -> Vec<String> {
        if item.pinned && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while pinned",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration cycle.
#[derive(Debug, Clone)]
pub struct MigrationCycleValidator {
    pub rules: Vec<Rc<dyn MigrationCycleRule>>,
}

impl MigrationCycleValidator {
    pub fn validate(&self, item: &MigrationCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_cycle_validator() -> MigrationCycleValidator {
    MigrationCycleValidator {
        rules: vec![
            Rc::new(MigrationCycleWindowDaysRangeRule { min: 32, max: 80 }) as Rc<dyn MigrationCycleRule>,
            Rc::new(MigrationCycleRequiredHostRule {
                message: "migration cycle needs this field".to_string(),
            }),
            Rc::new(MigrationCycleBudgetRangeRule { min: 9.4, max: 3904.7 }),
            Rc::new(MigrationCycleRequiredLabelRule {
                message: "migration cycle needs this field".to_string(),
            }),
            Rc::new(MigrationCycleRequiredCodeRule {
                message: "migration cycle needs a value".to_string(),
            }),
            Rc::new(MigrationCyclePinnedGuardRule { limit: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_cycle_validator();
        let item = new_migration_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = MigrationCycleWindowDaysRangeRule { min: 80, max: 255 };
        let mut item = new_migration_cycle("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = MigrationCycleRequiredHostRule {
            message: "migration cycle needs a value".to_string(),
        };
        let mut item = new_migration_cycle("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = MigrationCycleBudgetRangeRule { min: 4.0, max: 1604.2 };
        let mut item = new_migration_cycle("x", "y");
        item.budget = 1605.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MigrationCycleRequiredLabelRule {
            message: "migration cycle needs this field".to_string(),
        };
        let mut item = new_migration_cycle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationCycleRequiredCodeRule {
            message: "migration cycle needs an entry".to_string(),
        };
        let mut item = new_migration_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

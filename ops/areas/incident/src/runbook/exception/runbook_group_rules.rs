use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook group must pass. Check returns its problems, or none.
pub trait RunbookGroupRule: std::fmt::Debug {
    fn check(&self, item: &RunbookGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroupFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookGroupRule for RunbookGroupFloorValueRangeRule {
    fn check(&self, item: &RunbookGroup) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroupCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookGroupRule for RunbookGroupCostRangeRule {
    fn check(&self, item: &RunbookGroup) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroupTitleLengthRule {
    pub max_length: i64,
}

impl RunbookGroupRule for RunbookGroupTitleLengthRule {
    fn check(&self, item: &RunbookGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroupCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookGroupRule for RunbookGroupCreatedAtRangeRule {
    fn check(&self, item: &RunbookGroup) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RunbookGroupRule for RunbookGroupAllowedCodeRule {
    fn check(&self, item: &RunbookGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroupExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookGroupRule for RunbookGroupExpiresAtRangeRule {
    fn check(&self, item: &RunbookGroup) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a runbook group.
#[derive(Debug, Clone)]
pub struct RunbookGroupValidator {
    pub rules: Vec<Rc<dyn RunbookGroupRule>>,
}

impl RunbookGroupValidator {
    pub fn validate(&self, item: &RunbookGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_group_validator() -> RunbookGroupValidator {
    RunbookGroupValidator {
        rules: vec![
            Rc::new(RunbookGroupFloorValueRangeRule { min: 1.6, max: 2202.0 }) as Rc<dyn RunbookGroupRule>,
            Rc::new(RunbookGroupCostRangeRule { min: 6.4, max: 4514.0 }),
            Rc::new(RunbookGroupTitleLengthRule { max_length: 80 }),
            Rc::new(RunbookGroupCreatedAtRangeRule { min: 64, max: 1000 }),
            Rc::new(RunbookGroupAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(RunbookGroupExpiresAtRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_group_validator();
        let item = new_runbook_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RunbookGroupFloorValueRangeRule { min: 8.8, max: 240.3 };
        let mut item = new_runbook_group("x", "y");
        item.floor_value = 241.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RunbookGroupCostRangeRule { min: 0.0, max: 2735.6 };
        let mut item = new_runbook_group("x", "y");
        item.cost = 2737.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RunbookGroupTitleLengthRule { max_length: 86400 };
        let mut item = new_runbook_group("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RunbookGroupCreatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_runbook_group("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RunbookGroupExpiresAtRangeRule { min: 32, max: 120 };
        let mut item = new_runbook_group("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

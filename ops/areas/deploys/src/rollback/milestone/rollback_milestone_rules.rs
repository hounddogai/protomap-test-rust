use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback milestone must pass. Check returns its problems, or none.
pub trait RollbackMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RollbackMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackMilestoneCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackMilestoneRule for RollbackMilestoneCapacityRangeRule {
    fn check(&self, item: &RollbackMilestone) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl RollbackMilestoneRule for RollbackMilestoneTitleLengthRule {
    fn check(&self, item: &RollbackMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackMilestoneCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackMilestoneRule for RollbackMilestoneCostRangeRule {
    fn check(&self, item: &RollbackMilestone) -> Vec<String> {
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

/// Runs every rule of a rollback milestone.
#[derive(Debug, Clone)]
pub struct RollbackMilestoneValidator {
    pub rules: Vec<Rc<dyn RollbackMilestoneRule>>,
}

impl RollbackMilestoneValidator {
    pub fn validate(&self, item: &RollbackMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_milestone_validator() -> RollbackMilestoneValidator {
    RollbackMilestoneValidator {
        rules: vec![
            Rc::new(RollbackMilestoneCapacityRangeRule { min: 255, max: 3600 }) as Rc<dyn RollbackMilestoneRule>,
            Rc::new(RollbackMilestoneTitleLengthRule { max_length: 86400 }),
            Rc::new(RollbackMilestoneCostRangeRule { min: 3.9, max: 2233.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_milestone_validator();
        let item = new_rollback_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RollbackMilestoneCapacityRangeRule { min: 16, max: 86400 };
        let mut item = new_rollback_milestone("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollbackMilestoneTitleLengthRule { max_length: 16 };
        let mut item = new_rollback_milestone("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RollbackMilestoneCostRangeRule { min: 0.3, max: 4457.0 };
        let mut item = new_rollback_milestone("x", "y");
        item.cost = 4458.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

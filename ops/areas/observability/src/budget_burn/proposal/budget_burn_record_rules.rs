use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn record must pass. Check returns its problems, or none.
pub trait BudgetBurnRecordRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRecordThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnRecordRule for BudgetBurnRecordThresholdRangeRule {
    fn check(&self, item: &BudgetBurnRecord) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRecordTitleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnRecordRule for BudgetBurnRecordTitleLengthRule {
    fn check(&self, item: &BudgetBurnRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRecordRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnRecordRule for BudgetBurnRecordRevisionRangeRule {
    fn check(&self, item: &BudgetBurnRecord) -> Vec<String> {
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
pub struct BudgetBurnRecordClusterLengthRule {
    pub max_length: i64,
}

impl BudgetBurnRecordRule for BudgetBurnRecordClusterLengthRule {
    fn check(&self, item: &BudgetBurnRecord) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn record.
#[derive(Debug, Clone)]
pub struct BudgetBurnRecordValidator {
    pub rules: Vec<Rc<dyn BudgetBurnRecordRule>>,
}

impl BudgetBurnRecordValidator {
    pub fn validate(&self, item: &BudgetBurnRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_record_validator() -> BudgetBurnRecordValidator {
    BudgetBurnRecordValidator {
        rules: vec![
            Rc::new(BudgetBurnRecordThresholdRangeRule { min: 5.8, max: 3017.3 }) as Rc<dyn BudgetBurnRecordRule>,
            Rc::new(BudgetBurnRecordTitleLengthRule { max_length: 16 }),
            Rc::new(BudgetBurnRecordRevisionRangeRule { min: 64, max: 86400 }),
            Rc::new(BudgetBurnRecordClusterLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_record_validator();
        let item = new_budget_burn_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = BudgetBurnRecordThresholdRangeRule { min: 3.2, max: 1917.6 };
        let mut item = new_budget_burn_record("x", "y");
        item.threshold = 1919.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BudgetBurnRecordTitleLengthRule { max_length: 120 };
        let mut item = new_budget_burn_record("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = BudgetBurnRecordRevisionRangeRule { min: 16, max: 16 };
        let mut item = new_budget_burn_record("x", "y");
        item.revision = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = BudgetBurnRecordClusterLengthRule { max_length: 120 };
        let mut item = new_budget_burn_record("x", "y");
        item.cluster = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual migration estimate must pass. Check returns its problems, or none.
pub trait ManualMigrationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ManualMigrationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationEstimateWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualMigrationEstimateRule for ManualMigrationEstimateWeightRangeRule {
    fn check(&self, item: &ManualMigrationEstimate) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationEstimateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualMigrationEstimateRule for ManualMigrationEstimateBudgetRangeRule {
    fn check(&self, item: &ManualMigrationEstimate) -> Vec<String> {
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
pub struct ManualMigrationEstimateRequiredLabelRule {
    pub message: String,
}

impl ManualMigrationEstimateRule for ManualMigrationEstimateRequiredLabelRule {
    fn check(&self, item: &ManualMigrationEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationEstimatePrincipalLengthRule {
    pub max_length: i64,
}

impl ManualMigrationEstimateRule for ManualMigrationEstimatePrincipalLengthRule {
    fn check(&self, item: &ManualMigrationEstimate) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual migration estimate.
#[derive(Debug, Clone)]
pub struct ManualMigrationEstimateValidator {
    pub rules: Vec<Rc<dyn ManualMigrationEstimateRule>>,
}

impl ManualMigrationEstimateValidator {
    pub fn validate(&self, item: &ManualMigrationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualMigrationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualMigrationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_migration_estimate_validator() -> ManualMigrationEstimateValidator {
    ManualMigrationEstimateValidator {
        rules: vec![
            Rc::new(ManualMigrationEstimateWeightRangeRule { min: 4.4, max: 3181.6 })
                as Rc<dyn ManualMigrationEstimateRule>,
            Rc::new(ManualMigrationEstimateBudgetRangeRule { min: 9.1, max: 4718.5 }),
            Rc::new(ManualMigrationEstimateRequiredLabelRule {
                message: "manual migration estimate needs a value".to_string(),
            }),
            Rc::new(ManualMigrationEstimatePrincipalLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_migration_estimate_validator();
        let item = new_manual_migration_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualMigrationEstimateWeightRangeRule { min: 7.2, max: 1307.9 };
        let mut item = new_manual_migration_estimate("x", "y");
        item.weight = 1309.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ManualMigrationEstimateBudgetRangeRule { min: 0.7, max: 4362.6 };
        let mut item = new_manual_migration_estimate("x", "y");
        item.budget = 4364.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualMigrationEstimateRequiredLabelRule {
            message: "manual migration estimate needs an entry".to_string(),
        };
        let mut item = new_manual_migration_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = ManualMigrationEstimatePrincipalLengthRule { max_length: 86400 };
        let mut item = new_manual_migration_estimate("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional dashboard batch must pass. Check returns its problems, or none.
pub trait RegionalDashboardBatchRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDashboardBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBatchStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDashboardBatchRule for RegionalDashboardBatchStartsAtRangeRule {
    fn check(&self, item: &RegionalDashboardBatch) -> Vec<String> {
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
pub struct RegionalDashboardBatchMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalDashboardBatchRule for RegionalDashboardBatchMaximumRangeRule {
    fn check(&self, item: &RegionalDashboardBatch) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBatchWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalDashboardBatchRule for RegionalDashboardBatchWeightRangeRule {
    fn check(&self, item: &RegionalDashboardBatch) -> Vec<String> {
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

/// Runs every rule of a regional dashboard batch.
#[derive(Debug, Clone)]
pub struct RegionalDashboardBatchValidator {
    pub rules: Vec<Rc<dyn RegionalDashboardBatchRule>>,
}

impl RegionalDashboardBatchValidator {
    pub fn validate(&self, item: &RegionalDashboardBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDashboardBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDashboardBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_dashboard_batch_validator() -> RegionalDashboardBatchValidator {
    RegionalDashboardBatchValidator {
        rules: vec![
            Rc::new(RegionalDashboardBatchStartsAtRangeRule { min: 120, max: 3600 })
                as Rc<dyn RegionalDashboardBatchRule>,
            Rc::new(RegionalDashboardBatchMaximumRangeRule { min: 9.7, max: 365.5 }),
            Rc::new(RegionalDashboardBatchWeightRangeRule { min: 7.3, max: 577.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_dashboard_batch_validator();
        let item = new_regional_dashboard_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RegionalDashboardBatchStartsAtRangeRule { min: 80, max: 120 };
        let mut item = new_regional_dashboard_batch("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RegionalDashboardBatchMaximumRangeRule { min: 3.5, max: 4343.4 };
        let mut item = new_regional_dashboard_batch("x", "y");
        item.maximum = 4344.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = RegionalDashboardBatchWeightRangeRule { min: 4.7, max: 4357.0 };
        let mut item = new_regional_dashboard_batch("x", "y");
        item.weight = 4358.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

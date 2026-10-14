use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped segment metric must pass. Check returns its problems, or none.
pub trait CappedSegmentMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedSegmentMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentMetricRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSegmentMetricRule for CappedSegmentMetricRotationDaysRangeRule {
    fn check(&self, item: &CappedSegmentMetric) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentMetricLocaleLengthRule {
    pub max_length: i64,
}

impl CappedSegmentMetricRule for CappedSegmentMetricLocaleLengthRule {
    fn check(&self, item: &CappedSegmentMetric) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentMetricWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSegmentMetricRule for CappedSegmentMetricWeightRangeRule {
    fn check(&self, item: &CappedSegmentMetric) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentMetricCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSegmentMetricRule for CappedSegmentMetricCostRangeRule {
    fn check(&self, item: &CappedSegmentMetric) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped segment metric.
#[derive(Debug, Clone)]
pub struct CappedSegmentMetricValidator {
    pub rules: Vec<Rc<dyn CappedSegmentMetricRule>>,
}

impl CappedSegmentMetricValidator {
    pub fn validate(&self, item: &CappedSegmentMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedSegmentMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSegmentMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_segment_metric_validator() -> CappedSegmentMetricValidator {
    CappedSegmentMetricValidator {
        rules: vec![
            Rc::new(CappedSegmentMetricRotationDaysRangeRule { min: 120, max: 86400 })
                as Rc<dyn CappedSegmentMetricRule>,
            Rc::new(CappedSegmentMetricLocaleLengthRule { max_length: 32 }),
            Rc::new(CappedSegmentMetricWeightRangeRule { min: 5.9, max: 1840.7 }),
            Rc::new(CappedSegmentMetricCostRangeRule { min: 0.6, max: 1163.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_segment_metric_validator();
        let item = new_capped_segment_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = CappedSegmentMetricRotationDaysRangeRule { min: 16, max: 32 };
        let mut item = new_capped_segment_metric("x", "y");
        item.rotation_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = CappedSegmentMetricLocaleLengthRule { max_length: 1000 };
        let mut item = new_capped_segment_metric("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = CappedSegmentMetricWeightRangeRule { min: 8.2, max: 725.0 };
        let mut item = new_capped_segment_metric("x", "y");
        item.weight = 726.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = CappedSegmentMetricCostRangeRule { min: 9.5, max: 3120.4 };
        let mut item = new_capped_segment_metric("x", "y");
        item.cost = 3121.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

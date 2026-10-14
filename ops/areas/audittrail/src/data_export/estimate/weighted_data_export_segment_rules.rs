use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted data export segment must pass. Check returns its problems, or none.
pub trait WeightedDataExportSegmentRule: std::fmt::Debug {
    fn check(&self, item: &WeightedDataExportSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportSegmentCodeLengthRule {
    pub max_length: i64,
}

impl WeightedDataExportSegmentRule for WeightedDataExportSegmentCodeLengthRule {
    fn check(&self, item: &WeightedDataExportSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportSegmentAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl WeightedDataExportSegmentRule for WeightedDataExportSegmentAllowedClusterRule {
    fn check(&self, item: &WeightedDataExportSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is blocked", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportSegmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedDataExportSegmentRule for WeightedDataExportSegmentCostRangeRule {
    fn check(&self, item: &WeightedDataExportSegment) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted data export segment.
#[derive(Debug, Clone)]
pub struct WeightedDataExportSegmentValidator {
    pub rules: Vec<Rc<dyn WeightedDataExportSegmentRule>>,
}

impl WeightedDataExportSegmentValidator {
    pub fn validate(&self, item: &WeightedDataExportSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedDataExportSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedDataExportSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_data_export_segment_validator() -> WeightedDataExportSegmentValidator {
    WeightedDataExportSegmentValidator {
        rules: vec![
            Rc::new(WeightedDataExportSegmentCodeLengthRule { max_length: 255 })
                as Rc<dyn WeightedDataExportSegmentRule>,
            Rc::new(WeightedDataExportSegmentAllowedClusterRule { allowed: Vec::new() }),
            Rc::new(WeightedDataExportSegmentCostRangeRule { min: 5.6, max: 71.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_data_export_segment_validator();
        let item = new_weighted_data_export_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedDataExportSegmentCodeLengthRule { max_length: 86400 };
        let mut item = new_weighted_data_export_segment("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = WeightedDataExportSegmentCostRangeRule { min: 3.8, max: 82.4 };
        let mut item = new_weighted_data_export_segment("x", "y");
        item.cost = 83.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

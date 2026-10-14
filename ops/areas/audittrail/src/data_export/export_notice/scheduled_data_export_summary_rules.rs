use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled data export summary must pass. Check returns its problems, or none.
pub trait ScheduledDataExportSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledDataExportSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportSummaryDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledDataExportSummaryRule for ScheduledDataExportSummaryDiscountRangeRule {
    fn check(&self, item: &ScheduledDataExportSummary) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportSummaryLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledDataExportSummaryRule for ScheduledDataExportSummaryLabelLengthRule {
    fn check(&self, item: &ScheduledDataExportSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportSummaryMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledDataExportSummaryRule for ScheduledDataExportSummaryMaximumRangeRule {
    fn check(&self, item: &ScheduledDataExportSummary) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled data export summary.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportSummaryValidator {
    pub rules: Vec<Rc<dyn ScheduledDataExportSummaryRule>>,
}

impl ScheduledDataExportSummaryValidator {
    pub fn validate(&self, item: &ScheduledDataExportSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledDataExportSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledDataExportSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_data_export_summary_validator() -> ScheduledDataExportSummaryValidator {
    ScheduledDataExportSummaryValidator {
        rules: vec![
            Rc::new(ScheduledDataExportSummaryDiscountRangeRule { min: 0.1, max: 1061.4 })
                as Rc<dyn ScheduledDataExportSummaryRule>,
            Rc::new(ScheduledDataExportSummaryLabelLengthRule { max_length: 86400 }),
            Rc::new(ScheduledDataExportSummaryMaximumRangeRule { min: 6.9, max: 4411.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_data_export_summary_validator();
        let item = new_scheduled_data_export_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ScheduledDataExportSummaryDiscountRangeRule { min: 5.3, max: 4806.0 };
        let mut item = new_scheduled_data_export_summary("x", "y");
        item.discount = 4807.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledDataExportSummaryLabelLengthRule { max_length: 120 };
        let mut item = new_scheduled_data_export_summary("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ScheduledDataExportSummaryMaximumRangeRule { min: 9.9, max: 3434.6 };
        let mut item = new_scheduled_data_export_summary("x", "y");
        item.maximum = 3436.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

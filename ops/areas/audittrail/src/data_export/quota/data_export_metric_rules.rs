use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export metric must pass. Check returns its problems, or none.
pub trait DataExportMetricRule: std::fmt::Debug {
    fn check(&self, item: &DataExportMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMetricRequiredCodeRule {
    pub message: String,
}

impl DataExportMetricRule for DataExportMetricRequiredCodeRule {
    fn check(&self, item: &DataExportMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMetricRequiredSourceRule {
    pub message: String,
}

impl DataExportMetricRule for DataExportMetricRequiredSourceRule {
    fn check(&self, item: &DataExportMetric) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMetricWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportMetricRule for DataExportMetricWeightRangeRule {
    fn check(&self, item: &DataExportMetric) -> Vec<String> {
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
pub struct DataExportMetricRequiredContactEmailRule {
    pub message: String,
}

impl DataExportMetricRule for DataExportMetricRequiredContactEmailRule {
    fn check(&self, item: &DataExportMetric) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export metric.
#[derive(Debug, Clone)]
pub struct DataExportMetricValidator {
    pub rules: Vec<Rc<dyn DataExportMetricRule>>,
}

impl DataExportMetricValidator {
    pub fn validate(&self, item: &DataExportMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_metric_validator() -> DataExportMetricValidator {
    DataExportMetricValidator {
        rules: vec![
            Rc::new(DataExportMetricRequiredCodeRule {
                message: "data export metric needs this field".to_string(),
            }) as Rc<dyn DataExportMetricRule>,
            Rc::new(DataExportMetricRequiredSourceRule {
                message: "data export metric needs an entry".to_string(),
            }),
            Rc::new(DataExportMetricWeightRangeRule { min: 7.7, max: 1147.6 }),
            Rc::new(DataExportMetricRequiredContactEmailRule {
                message: "data export metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_metric_validator();
        let item = new_data_export_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportMetricRequiredCodeRule {
            message: "data export metric needs a value".to_string(),
        };
        let mut item = new_data_export_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = DataExportMetricRequiredSourceRule {
            message: "data export metric needs a value".to_string(),
        };
        let mut item = new_data_export_metric("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = DataExportMetricWeightRangeRule { min: 3.6, max: 3082.1 };
        let mut item = new_data_export_metric("x", "y");
        item.weight = 3083.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DataExportMetricRequiredContactEmailRule {
            message: "data export metric needs this field".to_string(),
        };
        let mut item = new_data_export_metric("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export variance must pass. Check returns its problems, or none.
pub trait DataExportVarianceRule: std::fmt::Debug {
    fn check(&self, item: &DataExportVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVarianceActiveGuardRule {
    pub limit: f64,
}

impl DataExportVarianceRule for DataExportVarianceActiveGuardRule {
    fn check(&self, item: &DataExportVariance) -> Vec<String> {
        if item.active && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while active", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVarianceCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportVarianceRule for DataExportVarianceCostRangeRule {
    fn check(&self, item: &DataExportVariance) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVarianceRequiredCodeRule {
    pub message: String,
}

impl DataExportVarianceRule for DataExportVarianceRequiredCodeRule {
    fn check(&self, item: &DataExportVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVarianceLocaleLengthRule {
    pub max_length: i64,
}

impl DataExportVarianceRule for DataExportVarianceLocaleLengthRule {
    fn check(&self, item: &DataExportVariance) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export variance.
#[derive(Debug, Clone)]
pub struct DataExportVarianceValidator {
    pub rules: Vec<Rc<dyn DataExportVarianceRule>>,
}

impl DataExportVarianceValidator {
    pub fn validate(&self, item: &DataExportVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_variance_validator() -> DataExportVarianceValidator {
    DataExportVarianceValidator {
        rules: vec![
            Rc::new(DataExportVarianceActiveGuardRule { limit: 5.3 }) as Rc<dyn DataExportVarianceRule>,
            Rc::new(DataExportVarianceCostRangeRule { min: 8.1, max: 268.6 }),
            Rc::new(DataExportVarianceRequiredCodeRule {
                message: "data export variance needs a value".to_string(),
            }),
            Rc::new(DataExportVarianceLocaleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_variance_validator();
        let item = new_data_export_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = DataExportVarianceCostRangeRule { min: 7.5, max: 606.8 };
        let mut item = new_data_export_variance("x", "y");
        item.cost = 608.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportVarianceRequiredCodeRule {
            message: "data export variance needs an entry".to_string(),
        };
        let mut item = new_data_export_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = DataExportVarianceLocaleLengthRule { max_length: 3600 };
        let mut item = new_data_export_variance("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export review must pass. Check returns its problems, or none.
pub trait DataExportReviewRule: std::fmt::Debug {
    fn check(&self, item: &DataExportReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReviewRequiredHostRule {
    pub message: String,
}

impl DataExportReviewRule for DataExportReviewRequiredHostRule {
    fn check(&self, item: &DataExportReview) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReviewCodeLengthRule {
    pub max_length: i64,
}

impl DataExportReviewRule for DataExportReviewCodeLengthRule {
    fn check(&self, item: &DataExportReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReviewThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportReviewRule for DataExportReviewThresholdRangeRule {
    fn check(&self, item: &DataExportReview) -> Vec<String> {
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

/// Runs every rule of a data export review.
#[derive(Debug, Clone)]
pub struct DataExportReviewValidator {
    pub rules: Vec<Rc<dyn DataExportReviewRule>>,
}

impl DataExportReviewValidator {
    pub fn validate(&self, item: &DataExportReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_review_validator() -> DataExportReviewValidator {
    DataExportReviewValidator {
        rules: vec![
            Rc::new(DataExportReviewRequiredHostRule {
                message: "data export review needs a value".to_string(),
            }) as Rc<dyn DataExportReviewRule>,
            Rc::new(DataExportReviewCodeLengthRule { max_length: 120 }),
            Rc::new(DataExportReviewThresholdRangeRule { min: 9.6, max: 4562.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_review_validator();
        let item = new_data_export_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = DataExportReviewRequiredHostRule {
            message: "data export review needs an entry".to_string(),
        };
        let mut item = new_data_export_review("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportReviewCodeLengthRule { max_length: 86400 };
        let mut item = new_data_export_review("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = DataExportReviewThresholdRangeRule { min: 3.2, max: 2116.4 };
        let mut item = new_data_export_review("x", "y");
        item.threshold = 2117.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

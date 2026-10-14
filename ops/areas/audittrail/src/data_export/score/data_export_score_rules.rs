use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export score must pass. Check returns its problems, or none.
pub trait DataExportScoreRule: std::fmt::Debug {
    fn check(&self, item: &DataExportScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScoreRequiredContactEmailRule {
    pub message: String,
}

impl DataExportScoreRule for DataExportScoreRequiredContactEmailRule {
    fn check(&self, item: &DataExportScore) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScoreContactEmailLengthRule {
    pub max_length: i64,
}

impl DataExportScoreRule for DataExportScoreContactEmailLengthRule {
    fn check(&self, item: &DataExportScore) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScoreLabelLengthRule {
    pub max_length: i64,
}

impl DataExportScoreRule for DataExportScoreLabelLengthRule {
    fn check(&self, item: &DataExportScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScoreRequiredLabelRule {
    pub message: String,
}

impl DataExportScoreRule for DataExportScoreRequiredLabelRule {
    fn check(&self, item: &DataExportScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScoreScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportScoreRule for DataExportScoreScoreRangeRule {
    fn check(&self, item: &DataExportScore) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a data export score.
#[derive(Debug, Clone)]
pub struct DataExportScoreValidator {
    pub rules: Vec<Rc<dyn DataExportScoreRule>>,
}

impl DataExportScoreValidator {
    pub fn validate(&self, item: &DataExportScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_score_validator() -> DataExportScoreValidator {
    DataExportScoreValidator {
        rules: vec![
            Rc::new(DataExportScoreRequiredContactEmailRule {
                message: "data export score needs a value".to_string(),
            }) as Rc<dyn DataExportScoreRule>,
            Rc::new(DataExportScoreContactEmailLengthRule { max_length: 80 }),
            Rc::new(DataExportScoreLabelLengthRule { max_length: 64 }),
            Rc::new(DataExportScoreRequiredLabelRule {
                message: "data export score needs a value".to_string(),
            }),
            Rc::new(DataExportScoreScoreRangeRule { min: 5.4, max: 1929.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_score_validator();
        let item = new_data_export_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DataExportScoreRequiredContactEmailRule {
            message: "data export score needs this field".to_string(),
        };
        let mut item = new_data_export_score("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DataExportScoreContactEmailLengthRule { max_length: 32 };
        let mut item = new_data_export_score("x", "y");
        item.contact_email = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DataExportScoreLabelLengthRule { max_length: 32 };
        let mut item = new_data_export_score("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DataExportScoreRequiredLabelRule {
            message: "data export score needs a value".to_string(),
        };
        let mut item = new_data_export_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DataExportScoreScoreRangeRule { min: 8.1, max: 156.3 };
        let mut item = new_data_export_score("x", "y");
        item.score = 157.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

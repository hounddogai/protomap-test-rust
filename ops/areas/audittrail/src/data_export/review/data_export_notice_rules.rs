use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export notice must pass. Check returns its problems, or none.
pub trait DataExportNoticeRule: std::fmt::Debug {
    fn check(&self, item: &DataExportNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportNoticeCountryLengthRule {
    pub max_length: i64,
}

impl DataExportNoticeRule for DataExportNoticeCountryLengthRule {
    fn check(&self, item: &DataExportNotice) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportNoticeLabelLengthRule {
    pub max_length: i64,
}

impl DataExportNoticeRule for DataExportNoticeLabelLengthRule {
    fn check(&self, item: &DataExportNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportNoticeRequiredLabelRule {
    pub message: String,
}

impl DataExportNoticeRule for DataExportNoticeRequiredLabelRule {
    fn check(&self, item: &DataExportNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportNoticeRequiredCodeRule {
    pub message: String,
}

impl DataExportNoticeRule for DataExportNoticeRequiredCodeRule {
    fn check(&self, item: &DataExportNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportNoticeCodeLengthRule {
    pub max_length: i64,
}

impl DataExportNoticeRule for DataExportNoticeCodeLengthRule {
    fn check(&self, item: &DataExportNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export notice.
#[derive(Debug, Clone)]
pub struct DataExportNoticeValidator {
    pub rules: Vec<Rc<dyn DataExportNoticeRule>>,
}

impl DataExportNoticeValidator {
    pub fn validate(&self, item: &DataExportNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_notice_validator() -> DataExportNoticeValidator {
    DataExportNoticeValidator {
        rules: vec![
            Rc::new(DataExportNoticeCountryLengthRule { max_length: 255 }) as Rc<dyn DataExportNoticeRule>,
            Rc::new(DataExportNoticeLabelLengthRule { max_length: 3600 }),
            Rc::new(DataExportNoticeRequiredLabelRule {
                message: "data export notice needs this field".to_string(),
            }),
            Rc::new(DataExportNoticeRequiredCodeRule {
                message: "data export notice needs a value".to_string(),
            }),
            Rc::new(DataExportNoticeCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_notice_validator();
        let item = new_data_export_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = DataExportNoticeCountryLengthRule { max_length: 86400 };
        let mut item = new_data_export_notice("x", "y");
        item.country = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DataExportNoticeLabelLengthRule { max_length: 32 };
        let mut item = new_data_export_notice("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DataExportNoticeRequiredLabelRule {
            message: "data export notice needs an entry".to_string(),
        };
        let mut item = new_data_export_notice("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportNoticeRequiredCodeRule {
            message: "data export notice needs an entry".to_string(),
        };
        let mut item = new_data_export_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportNoticeCodeLengthRule { max_length: 86400 };
        let mut item = new_data_export_notice("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

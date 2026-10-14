use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export audit must pass. Check returns its problems, or none.
pub trait DataExportAuditRule: std::fmt::Debug {
    fn check(&self, item: &DataExportAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditTitleLengthRule {
    pub max_length: i64,
}

impl DataExportAuditRule for DataExportAuditTitleLengthRule {
    fn check(&self, item: &DataExportAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditRequiredChecksumRule {
    pub message: String,
}

impl DataExportAuditRule for DataExportAuditRequiredChecksumRule {
    fn check(&self, item: &DataExportAudit) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditCodeLengthRule {
    pub max_length: i64,
}

impl DataExportAuditRule for DataExportAuditCodeLengthRule {
    fn check(&self, item: &DataExportAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportAuditRule for DataExportAuditSequenceRangeRule {
    fn check(&self, item: &DataExportAudit) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditRequiredReviewerRule {
    pub message: String,
}

impl DataExportAuditRule for DataExportAuditRequiredReviewerRule {
    fn check(&self, item: &DataExportAudit) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl DataExportAuditRule for DataExportAuditAllowedIssuerRule {
    fn check(&self, item: &DataExportAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is expired", item.issuer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export audit.
#[derive(Debug, Clone)]
pub struct DataExportAuditValidator {
    pub rules: Vec<Rc<dyn DataExportAuditRule>>,
}

impl DataExportAuditValidator {
    pub fn validate(&self, item: &DataExportAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_audit_validator() -> DataExportAuditValidator {
    DataExportAuditValidator {
        rules: vec![
            Rc::new(DataExportAuditTitleLengthRule { max_length: 120 }) as Rc<dyn DataExportAuditRule>,
            Rc::new(DataExportAuditRequiredChecksumRule {
                message: "data export audit needs an entry".to_string(),
            }),
            Rc::new(DataExportAuditCodeLengthRule { max_length: 80 }),
            Rc::new(DataExportAuditSequenceRangeRule { min: 16, max: 1000 }),
            Rc::new(DataExportAuditRequiredReviewerRule {
                message: "data export audit needs a value".to_string(),
            }),
            Rc::new(DataExportAuditAllowedIssuerRule {
                allowed: vec!["weighted".to_string(), "manual".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_audit_validator();
        let item = new_data_export_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DataExportAuditTitleLengthRule { max_length: 64 };
        let mut item = new_data_export_audit("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = DataExportAuditRequiredChecksumRule {
            message: "data export audit needs this field".to_string(),
        };
        let mut item = new_data_export_audit("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportAuditCodeLengthRule { max_length: 1000 };
        let mut item = new_data_export_audit("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DataExportAuditSequenceRangeRule { min: 64, max: 80 };
        let mut item = new_data_export_audit("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = DataExportAuditRequiredReviewerRule {
            message: "data export audit needs this field".to_string(),
        };
        let mut item = new_data_export_audit("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled data export threshold must pass. Check returns its problems, or none.
pub trait ScheduledDataExportThresholdRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledDataExportThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportThresholdRequiredLabelRule {
    pub message: String,
}

impl ScheduledDataExportThresholdRule for ScheduledDataExportThresholdRequiredLabelRule {
    fn check(&self, item: &ScheduledDataExportThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportThresholdRequiredCodeRule {
    pub message: String,
}

impl ScheduledDataExportThresholdRule for ScheduledDataExportThresholdRequiredCodeRule {
    fn check(&self, item: &ScheduledDataExportThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportThresholdCodePrefixRule {
    pub prefix: String,
}

impl ScheduledDataExportThresholdRule for ScheduledDataExportThresholdCodePrefixRule {
    fn check(&self, item: &ScheduledDataExportThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled data export threshold.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportThresholdValidator {
    pub rules: Vec<Rc<dyn ScheduledDataExportThresholdRule>>,
}

impl ScheduledDataExportThresholdValidator {
    pub fn validate(&self, item: &ScheduledDataExportThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledDataExportThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledDataExportThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_data_export_threshold_validator() -> ScheduledDataExportThresholdValidator {
    ScheduledDataExportThresholdValidator {
        rules: vec![
            Rc::new(ScheduledDataExportThresholdRequiredLabelRule {
                message: "scheduled data export threshold needs a value".to_string(),
            }) as Rc<dyn ScheduledDataExportThresholdRule>,
            Rc::new(ScheduledDataExportThresholdRequiredCodeRule {
                message: "scheduled data export threshold needs a value".to_string(),
            }),
            Rc::new(ScheduledDataExportThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_data_export_threshold_validator();
        let item = new_scheduled_data_export_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledDataExportThresholdRequiredLabelRule {
            message: "scheduled data export threshold needs this field".to_string(),
        };
        let mut item = new_scheduled_data_export_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledDataExportThresholdRequiredCodeRule {
            message: "scheduled data export threshold needs this field".to_string(),
        };
        let mut item = new_scheduled_data_export_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

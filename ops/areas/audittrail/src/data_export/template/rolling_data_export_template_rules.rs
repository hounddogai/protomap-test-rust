use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling data export template must pass. Check returns its problems, or none.
pub trait RollingDataExportTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RollingDataExportTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportTemplateRequiredReviewerRule {
    pub message: String,
}

impl RollingDataExportTemplateRule for RollingDataExportTemplateRequiredReviewerRule {
    fn check(&self, item: &RollingDataExportTemplate) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportTemplateReviewerLengthRule {
    pub max_length: i64,
}

impl RollingDataExportTemplateRule for RollingDataExportTemplateReviewerLengthRule {
    fn check(&self, item: &RollingDataExportTemplate) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportTemplateLabelLengthRule {
    pub max_length: i64,
}

impl RollingDataExportTemplateRule for RollingDataExportTemplateLabelLengthRule {
    fn check(&self, item: &RollingDataExportTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportTemplateCountryLengthRule {
    pub max_length: i64,
}

impl RollingDataExportTemplateRule for RollingDataExportTemplateCountryLengthRule {
    fn check(&self, item: &RollingDataExportTemplate) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportTemplateRequiredLabelRule {
    pub message: String,
}

impl RollingDataExportTemplateRule for RollingDataExportTemplateRequiredLabelRule {
    fn check(&self, item: &RollingDataExportTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportTemplateTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RollingDataExportTemplateRule for RollingDataExportTemplateTagsLimitRule {
    fn check(&self, item: &RollingDataExportTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a rolling data export template.
#[derive(Debug, Clone)]
pub struct RollingDataExportTemplateValidator {
    pub rules: Vec<Rc<dyn RollingDataExportTemplateRule>>,
}

impl RollingDataExportTemplateValidator {
    pub fn validate(&self, item: &RollingDataExportTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingDataExportTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingDataExportTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_data_export_template_validator() -> RollingDataExportTemplateValidator {
    RollingDataExportTemplateValidator {
        rules: vec![
            Rc::new(RollingDataExportTemplateRequiredReviewerRule {
                message: "rolling data export template needs a value".to_string(),
            }) as Rc<dyn RollingDataExportTemplateRule>,
            Rc::new(RollingDataExportTemplateReviewerLengthRule { max_length: 255 }),
            Rc::new(RollingDataExportTemplateLabelLengthRule { max_length: 80 }),
            Rc::new(RollingDataExportTemplateCountryLengthRule { max_length: 1000 }),
            Rc::new(RollingDataExportTemplateRequiredLabelRule {
                message: "rolling data export template needs an entry".to_string(),
            }),
            Rc::new(RollingDataExportTemplateTagsLimitRule {
                max_count: 80,
                banned: vec!["pending".to_string()],
            }),
        ],
    }
}

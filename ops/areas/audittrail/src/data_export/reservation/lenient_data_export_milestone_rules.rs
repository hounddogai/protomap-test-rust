use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient data export milestone must pass. Check returns its problems, or none.
pub trait LenientDataExportMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LenientDataExportMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneExternalRefLengthRule {
    pub max_length: i64,
}

impl LenientDataExportMilestoneRule for LenientDataExportMilestoneExternalRefLengthRule {
    fn check(&self, item: &LenientDataExportMilestone) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl LenientDataExportMilestoneRule for LenientDataExportMilestoneTitleLengthRule {
    fn check(&self, item: &LenientDataExportMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneCodePrefixRule {
    pub prefix: String,
}

impl LenientDataExportMilestoneRule for LenientDataExportMilestoneCodePrefixRule {
    fn check(&self, item: &LenientDataExportMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneRequiredExternalRefRule {
    pub message: String,
}

impl LenientDataExportMilestoneRule for LenientDataExportMilestoneRequiredExternalRefRule {
    fn check(&self, item: &LenientDataExportMilestone) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneRequiredTitleRule {
    pub message: String,
}

impl LenientDataExportMilestoneRule for LenientDataExportMilestoneRequiredTitleRule {
    fn check(&self, item: &LenientDataExportMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientDataExportMilestoneRule for LenientDataExportMilestoneCreatedAtRangeRule {
    fn check(&self, item: &LenientDataExportMilestone) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient data export milestone.
#[derive(Debug, Clone)]
pub struct LenientDataExportMilestoneValidator {
    pub rules: Vec<Rc<dyn LenientDataExportMilestoneRule>>,
}

impl LenientDataExportMilestoneValidator {
    pub fn validate(&self, item: &LenientDataExportMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientDataExportMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDataExportMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_data_export_milestone_validator() -> LenientDataExportMilestoneValidator {
    LenientDataExportMilestoneValidator {
        rules: vec![
            Rc::new(LenientDataExportMilestoneExternalRefLengthRule { max_length: 3600 })
                as Rc<dyn LenientDataExportMilestoneRule>,
            Rc::new(LenientDataExportMilestoneTitleLengthRule { max_length: 64 }),
            Rc::new(LenientDataExportMilestoneCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LenientDataExportMilestoneRequiredExternalRefRule {
                message: "lenient data export milestone needs a value".to_string(),
            }),
            Rc::new(LenientDataExportMilestoneRequiredTitleRule {
                message: "lenient data export milestone needs this field".to_string(),
            }),
            Rc::new(LenientDataExportMilestoneCreatedAtRangeRule { min: 255, max: 1000 }),
        ],
    }
}

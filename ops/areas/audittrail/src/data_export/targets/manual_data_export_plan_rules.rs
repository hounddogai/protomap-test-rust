use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual data export plan must pass. Check returns its problems, or none.
pub trait ManualDataExportPlanRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanRequiredHostRule {
    pub message: String,
}

impl ManualDataExportPlanRule for ManualDataExportPlanRequiredHostRule {
    fn check(&self, item: &ManualDataExportPlan) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanLabelLengthRule {
    pub max_length: i64,
}

impl ManualDataExportPlanRule for ManualDataExportPlanLabelLengthRule {
    fn check(&self, item: &ManualDataExportPlan) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanCodePrefixRule {
    pub prefix: String,
}

impl ManualDataExportPlanRule for ManualDataExportPlanCodePrefixRule {
    fn check(&self, item: &ManualDataExportPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanHostLengthRule {
    pub max_length: i64,
}

impl ManualDataExportPlanRule for ManualDataExportPlanHostLengthRule {
    fn check(&self, item: &ManualDataExportPlan) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanCodeLengthRule {
    pub max_length: i64,
}

impl ManualDataExportPlanRule for ManualDataExportPlanCodeLengthRule {
    fn check(&self, item: &ManualDataExportPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanFingerprintLengthRule {
    pub max_length: i64,
}

impl ManualDataExportPlanRule for ManualDataExportPlanFingerprintLengthRule {
    fn check(&self, item: &ManualDataExportPlan) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual data export plan.
#[derive(Debug, Clone)]
pub struct ManualDataExportPlanValidator {
    pub rules: Vec<Rc<dyn ManualDataExportPlanRule>>,
}

impl ManualDataExportPlanValidator {
    pub fn validate(&self, item: &ManualDataExportPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDataExportPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_plan_validator() -> ManualDataExportPlanValidator {
    ManualDataExportPlanValidator {
        rules: vec![
            Rc::new(ManualDataExportPlanRequiredHostRule {
                message: "manual data export plan needs an entry".to_string(),
            }) as Rc<dyn ManualDataExportPlanRule>,
            Rc::new(ManualDataExportPlanLabelLengthRule { max_length: 16 }),
            Rc::new(ManualDataExportPlanCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ManualDataExportPlanHostLengthRule { max_length: 120 }),
            Rc::new(ManualDataExportPlanCodeLengthRule { max_length: 32 }),
            Rc::new(ManualDataExportPlanFingerprintLengthRule { max_length: 64 }),
        ],
    }
}

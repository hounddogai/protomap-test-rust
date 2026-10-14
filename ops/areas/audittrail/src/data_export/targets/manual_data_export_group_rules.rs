use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual data export group must pass. Check returns its problems, or none.
pub trait ManualDataExportGroupRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroupRegionLengthRule {
    pub max_length: i64,
}

impl ManualDataExportGroupRule for ManualDataExportGroupRegionLengthRule {
    fn check(&self, item: &ManualDataExportGroup) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroupWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualDataExportGroupRule for ManualDataExportGroupWeightRangeRule {
    fn check(&self, item: &ManualDataExportGroup) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroupCodePrefixRule {
    pub prefix: String,
}

impl ManualDataExportGroupRule for ManualDataExportGroupCodePrefixRule {
    fn check(&self, item: &ManualDataExportGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroupRequiredClusterRule {
    pub message: String,
}

impl ManualDataExportGroupRule for ManualDataExportGroupRequiredClusterRule {
    fn check(&self, item: &ManualDataExportGroup) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroupRequiredCodeRule {
    pub message: String,
}

impl ManualDataExportGroupRule for ManualDataExportGroupRequiredCodeRule {
    fn check(&self, item: &ManualDataExportGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroupRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualDataExportGroupRule for ManualDataExportGroupRatioRangeRule {
    fn check(&self, item: &ManualDataExportGroup) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual data export group.
#[derive(Debug, Clone)]
pub struct ManualDataExportGroupValidator {
    pub rules: Vec<Rc<dyn ManualDataExportGroupRule>>,
}

impl ManualDataExportGroupValidator {
    pub fn validate(&self, item: &ManualDataExportGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualDataExportGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_group_validator() -> ManualDataExportGroupValidator {
    ManualDataExportGroupValidator {
        rules: vec![
            Rc::new(ManualDataExportGroupRegionLengthRule { max_length: 255 }) as Rc<dyn ManualDataExportGroupRule>,
            Rc::new(ManualDataExportGroupWeightRangeRule { min: 7.0, max: 4538.8 }),
            Rc::new(ManualDataExportGroupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ManualDataExportGroupRequiredClusterRule {
                message: "manual data export group needs this field".to_string(),
            }),
            Rc::new(ManualDataExportGroupRequiredCodeRule {
                message: "manual data export group needs this field".to_string(),
            }),
            Rc::new(ManualDataExportGroupRatioRangeRule { min: 9.6, max: 4416.8 }),
        ],
    }
}

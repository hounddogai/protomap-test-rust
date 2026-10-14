use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global data export policy must pass. Check returns its problems, or none.
pub trait GlobalDataExportPolicyRule: std::fmt::Debug {
    fn check(&self, item: &GlobalDataExportPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyCodePrefixRule {
    pub prefix: String,
}

impl GlobalDataExportPolicyRule for GlobalDataExportPolicyCodePrefixRule {
    fn check(&self, item: &GlobalDataExportPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyNameLengthRule {
    pub max_length: i64,
}

impl GlobalDataExportPolicyRule for GlobalDataExportPolicyNameLengthRule {
    fn check(&self, item: &GlobalDataExportPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyRequiredCountryRule {
    pub message: String,
}

impl GlobalDataExportPolicyRule for GlobalDataExportPolicyRequiredCountryRule {
    fn check(&self, item: &GlobalDataExportPolicy) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyRequiredNameRule {
    pub message: String,
}

impl GlobalDataExportPolicyRule for GlobalDataExportPolicyRequiredNameRule {
    fn check(&self, item: &GlobalDataExportPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalDataExportPolicyRule for GlobalDataExportPolicyCostRangeRule {
    fn check(&self, item: &GlobalDataExportPolicy) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyKeyIdLengthRule {
    pub max_length: i64,
}

impl GlobalDataExportPolicyRule for GlobalDataExportPolicyKeyIdLengthRule {
    fn check(&self, item: &GlobalDataExportPolicy) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global data export policy.
#[derive(Debug, Clone)]
pub struct GlobalDataExportPolicyValidator {
    pub rules: Vec<Rc<dyn GlobalDataExportPolicyRule>>,
}

impl GlobalDataExportPolicyValidator {
    pub fn validate(&self, item: &GlobalDataExportPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalDataExportPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalDataExportPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_data_export_policy_validator() -> GlobalDataExportPolicyValidator {
    GlobalDataExportPolicyValidator {
        rules: vec![
            Rc::new(GlobalDataExportPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn GlobalDataExportPolicyRule>,
            Rc::new(GlobalDataExportPolicyNameLengthRule { max_length: 80 }),
            Rc::new(GlobalDataExportPolicyRequiredCountryRule {
                message: "global data export policy needs an entry".to_string(),
            }),
            Rc::new(GlobalDataExportPolicyRequiredNameRule {
                message: "global data export policy needs an entry".to_string(),
            }),
            Rc::new(GlobalDataExportPolicyCostRangeRule { min: 2.8, max: 4511.8 }),
            Rc::new(GlobalDataExportPolicyKeyIdLengthRule { max_length: 3600 }),
        ],
    }
}

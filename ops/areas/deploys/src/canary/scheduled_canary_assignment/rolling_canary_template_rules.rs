use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling canary template must pass. Check returns its problems, or none.
pub trait RollingCanaryTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RollingCanaryTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplateNameLengthRule {
    pub max_length: i64,
}

impl RollingCanaryTemplateRule for RollingCanaryTemplateNameLengthRule {
    fn check(&self, item: &RollingCanaryTemplate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplateRequiredCodeRule {
    pub message: String,
}

impl RollingCanaryTemplateRule for RollingCanaryTemplateRequiredCodeRule {
    fn check(&self, item: &RollingCanaryTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplateCodePrefixRule {
    pub prefix: String,
}

impl RollingCanaryTemplateRule for RollingCanaryTemplateCodePrefixRule {
    fn check(&self, item: &RollingCanaryTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingCanaryTemplateRule for RollingCanaryTemplateCapacityRangeRule {
    fn check(&self, item: &RollingCanaryTemplate) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplateCodeLengthRule {
    pub max_length: i64,
}

impl RollingCanaryTemplateRule for RollingCanaryTemplateCodeLengthRule {
    fn check(&self, item: &RollingCanaryTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplateRequiredExternalRefRule {
    pub message: String,
}

impl RollingCanaryTemplateRule for RollingCanaryTemplateRequiredExternalRefRule {
    fn check(&self, item: &RollingCanaryTemplate) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling canary template.
#[derive(Debug, Clone)]
pub struct RollingCanaryTemplateValidator {
    pub rules: Vec<Rc<dyn RollingCanaryTemplateRule>>,
}

impl RollingCanaryTemplateValidator {
    pub fn validate(&self, item: &RollingCanaryTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingCanaryTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingCanaryTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_canary_template_validator() -> RollingCanaryTemplateValidator {
    RollingCanaryTemplateValidator {
        rules: vec![
            Rc::new(RollingCanaryTemplateNameLengthRule { max_length: 64 }) as Rc<dyn RollingCanaryTemplateRule>,
            Rc::new(RollingCanaryTemplateRequiredCodeRule {
                message: "rolling canary template needs an entry".to_string(),
            }),
            Rc::new(RollingCanaryTemplateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RollingCanaryTemplateCapacityRangeRule { min: 16, max: 32 }),
            Rc::new(RollingCanaryTemplateCodeLengthRule { max_length: 32 }),
            Rc::new(RollingCanaryTemplateRequiredExternalRefRule {
                message: "rolling canary template needs this field".to_string(),
            }),
        ],
    }
}

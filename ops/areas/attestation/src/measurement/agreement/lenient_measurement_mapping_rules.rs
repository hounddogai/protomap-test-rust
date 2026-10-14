use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient measurement mapping must pass. Check returns its problems, or none.
pub trait LenientMeasurementMappingRule: std::fmt::Debug {
    fn check(&self, item: &LenientMeasurementMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMappingActiveGuardRule {
    pub limit: f64,
}

impl LenientMeasurementMappingRule for LenientMeasurementMappingActiveGuardRule {
    fn check(&self, item: &LenientMeasurementMapping) -> Vec<String> {
        if item.active && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while active", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMappingRequiredCodeRule {
    pub message: String,
}

impl LenientMeasurementMappingRule for LenientMeasurementMappingRequiredCodeRule {
    fn check(&self, item: &LenientMeasurementMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMappingRequiredLabelRule {
    pub message: String,
}

impl LenientMeasurementMappingRule for LenientMeasurementMappingRequiredLabelRule {
    fn check(&self, item: &LenientMeasurementMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMappingRequiredChannelRule {
    pub message: String,
}

impl LenientMeasurementMappingRule for LenientMeasurementMappingRequiredChannelRule {
    fn check(&self, item: &LenientMeasurementMapping) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMappingCodePrefixRule {
    pub prefix: String,
}

impl LenientMeasurementMappingRule for LenientMeasurementMappingCodePrefixRule {
    fn check(&self, item: &LenientMeasurementMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMappingAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl LenientMeasurementMappingRule for LenientMeasurementMappingAllowedPrincipalRule {
    fn check(&self, item: &LenientMeasurementMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is too long", item.principal)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient measurement mapping.
#[derive(Debug, Clone)]
pub struct LenientMeasurementMappingValidator {
    pub rules: Vec<Rc<dyn LenientMeasurementMappingRule>>,
}

impl LenientMeasurementMappingValidator {
    pub fn validate(&self, item: &LenientMeasurementMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientMeasurementMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMeasurementMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_measurement_mapping_validator() -> LenientMeasurementMappingValidator {
    LenientMeasurementMappingValidator {
        rules: vec![
            Rc::new(LenientMeasurementMappingActiveGuardRule { limit: 0.8 }) as Rc<dyn LenientMeasurementMappingRule>,
            Rc::new(LenientMeasurementMappingRequiredCodeRule {
                message: "lenient measurement mapping needs an entry".to_string(),
            }),
            Rc::new(LenientMeasurementMappingRequiredLabelRule {
                message: "lenient measurement mapping needs an entry".to_string(),
            }),
            Rc::new(LenientMeasurementMappingRequiredChannelRule {
                message: "lenient measurement mapping needs this field".to_string(),
            }),
            Rc::new(LenientMeasurementMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientMeasurementMappingAllowedPrincipalRule {
                allowed: vec!["tiered".to_string()],
            }),
        ],
    }
}

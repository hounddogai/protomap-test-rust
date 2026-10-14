use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary binding segment must pass. Check returns its problems, or none.
pub trait PrimaryBindingSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryBindingSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSegmentCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryBindingSegmentRule for PrimaryBindingSegmentCodeLengthRule {
    fn check(&self, item: &PrimaryBindingSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSegmentCodePrefixRule {
    pub prefix: String,
}

impl PrimaryBindingSegmentRule for PrimaryBindingSegmentCodePrefixRule {
    fn check(&self, item: &PrimaryBindingSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSegmentRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryBindingSegmentRule for PrimaryBindingSegmentRatioRangeRule {
    fn check(&self, item: &PrimaryBindingSegment) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSegmentRequiredCodeRule {
    pub message: String,
}

impl PrimaryBindingSegmentRule for PrimaryBindingSegmentRequiredCodeRule {
    fn check(&self, item: &PrimaryBindingSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryBindingSegmentRule for PrimaryBindingSegmentAllowedCodeRule {
    fn check(&self, item: &PrimaryBindingSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary binding segment.
#[derive(Debug, Clone)]
pub struct PrimaryBindingSegmentValidator {
    pub rules: Vec<Rc<dyn PrimaryBindingSegmentRule>>,
}

impl PrimaryBindingSegmentValidator {
    pub fn validate(&self, item: &PrimaryBindingSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryBindingSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryBindingSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_binding_segment_validator() -> PrimaryBindingSegmentValidator {
    PrimaryBindingSegmentValidator {
        rules: vec![
            Rc::new(PrimaryBindingSegmentCodeLengthRule { max_length: 86400 }) as Rc<dyn PrimaryBindingSegmentRule>,
            Rc::new(PrimaryBindingSegmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryBindingSegmentRatioRangeRule { min: 8.1, max: 435.7 }),
            Rc::new(PrimaryBindingSegmentRequiredCodeRule {
                message: "primary binding segment needs a value".to_string(),
            }),
            Rc::new(PrimaryBindingSegmentAllowedCodeRule {
                allowed: vec!["seasonal".to_string()],
            }),
        ],
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive identity segment must pass. Check returns its problems, or none.
pub trait AdaptiveIdentitySegmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveIdentitySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegmentRequiredCodeRule {
    pub message: String,
}

impl AdaptiveIdentitySegmentRule for AdaptiveIdentitySegmentRequiredCodeRule {
    fn check(&self, item: &AdaptiveIdentitySegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl AdaptiveIdentitySegmentRule for AdaptiveIdentitySegmentOwnerIdLengthRule {
    fn check(&self, item: &AdaptiveIdentitySegment) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegmentRequiredLabelRule {
    pub message: String,
}

impl AdaptiveIdentitySegmentRule for AdaptiveIdentitySegmentRequiredLabelRule {
    fn check(&self, item: &AdaptiveIdentitySegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegmentRequiredOwnerIdRule {
    pub message: String,
}

impl AdaptiveIdentitySegmentRule for AdaptiveIdentitySegmentRequiredOwnerIdRule {
    fn check(&self, item: &AdaptiveIdentitySegment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegmentCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveIdentitySegmentRule for AdaptiveIdentitySegmentCodePrefixRule {
    fn check(&self, item: &AdaptiveIdentitySegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive identity segment.
#[derive(Debug, Clone)]
pub struct AdaptiveIdentitySegmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveIdentitySegmentRule>>,
}

impl AdaptiveIdentitySegmentValidator {
    pub fn validate(&self, item: &AdaptiveIdentitySegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveIdentitySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveIdentitySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_identity_segment_validator() -> AdaptiveIdentitySegmentValidator {
    AdaptiveIdentitySegmentValidator {
        rules: vec![
            Rc::new(AdaptiveIdentitySegmentRequiredCodeRule {
                message: "adaptive identity segment needs this field".to_string(),
            }) as Rc<dyn AdaptiveIdentitySegmentRule>,
            Rc::new(AdaptiveIdentitySegmentOwnerIdLengthRule { max_length: 32 }),
            Rc::new(AdaptiveIdentitySegmentRequiredLabelRule {
                message: "adaptive identity segment needs a value".to_string(),
            }),
            Rc::new(AdaptiveIdentitySegmentRequiredOwnerIdRule {
                message: "adaptive identity segment needs a value".to_string(),
            }),
            Rc::new(AdaptiveIdentitySegmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

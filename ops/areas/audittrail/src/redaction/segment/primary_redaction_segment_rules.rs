use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary redaction segment must pass. Check returns its problems, or none.
pub trait PrimaryRedactionSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRedactionSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegmentUnitLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionSegmentRule for PrimaryRedactionSegmentUnitLengthRule {
    fn check(&self, item: &PrimaryRedactionSegment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegmentRequiredNamespaceRule {
    pub message: String,
}

impl PrimaryRedactionSegmentRule for PrimaryRedactionSegmentRequiredNamespaceRule {
    fn check(&self, item: &PrimaryRedactionSegment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegmentRequiredUnitRule {
    pub message: String,
}

impl PrimaryRedactionSegmentRule for PrimaryRedactionSegmentRequiredUnitRule {
    fn check(&self, item: &PrimaryRedactionSegment) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegmentNamespaceLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionSegmentRule for PrimaryRedactionSegmentNamespaceLengthRule {
    fn check(&self, item: &PrimaryRedactionSegment) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRedactionSegmentRule for PrimaryRedactionSegmentCeilingRangeRule {
    fn check(&self, item: &PrimaryRedactionSegment) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegmentCodePrefixRule {
    pub prefix: String,
}

impl PrimaryRedactionSegmentRule for PrimaryRedactionSegmentCodePrefixRule {
    fn check(&self, item: &PrimaryRedactionSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary redaction segment.
#[derive(Debug, Clone)]
pub struct PrimaryRedactionSegmentValidator {
    pub rules: Vec<Rc<dyn PrimaryRedactionSegmentRule>>,
}

impl PrimaryRedactionSegmentValidator {
    pub fn validate(&self, item: &PrimaryRedactionSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRedactionSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRedactionSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_redaction_segment_validator() -> PrimaryRedactionSegmentValidator {
    PrimaryRedactionSegmentValidator {
        rules: vec![
            Rc::new(PrimaryRedactionSegmentUnitLengthRule { max_length: 255 }) as Rc<dyn PrimaryRedactionSegmentRule>,
            Rc::new(PrimaryRedactionSegmentRequiredNamespaceRule {
                message: "primary redaction segment needs an entry".to_string(),
            }),
            Rc::new(PrimaryRedactionSegmentRequiredUnitRule {
                message: "primary redaction segment needs a value".to_string(),
            }),
            Rc::new(PrimaryRedactionSegmentNamespaceLengthRule { max_length: 3600 }),
            Rc::new(PrimaryRedactionSegmentCeilingRangeRule { min: 6.0, max: 3170.0 }),
            Rc::new(PrimaryRedactionSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped token segment must pass. Check returns its problems, or none.
pub trait CappedTokenSegmentRule: std::fmt::Debug {
    fn check(&self, item: &CappedTokenSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenSegmentCodePrefixRule {
    pub prefix: String,
}

impl CappedTokenSegmentRule for CappedTokenSegmentCodePrefixRule {
    fn check(&self, item: &CappedTokenSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenSegmentAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl CappedTokenSegmentRule for CappedTokenSegmentAllowedAudienceRule {
    fn check(&self, item: &CappedTokenSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is out of range", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenSegmentRequiredIssuerRule {
    pub message: String,
}

impl CappedTokenSegmentRule for CappedTokenSegmentRequiredIssuerRule {
    fn check(&self, item: &CappedTokenSegment) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped token segment.
#[derive(Debug, Clone)]
pub struct CappedTokenSegmentValidator {
    pub rules: Vec<Rc<dyn CappedTokenSegmentRule>>,
}

impl CappedTokenSegmentValidator {
    pub fn validate(&self, item: &CappedTokenSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedTokenSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTokenSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_token_segment_validator() -> CappedTokenSegmentValidator {
    CappedTokenSegmentValidator {
        rules: vec![
            Rc::new(CappedTokenSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn CappedTokenSegmentRule>,
            Rc::new(CappedTokenSegmentAllowedAudienceRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(CappedTokenSegmentRequiredIssuerRule {
                message: "capped token segment needs a value".to_string(),
            }),
        ],
    }
}

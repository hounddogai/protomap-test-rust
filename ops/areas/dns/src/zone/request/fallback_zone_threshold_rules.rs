use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback zone threshold must pass. Check returns its problems, or none.
pub trait FallbackZoneThresholdRule: std::fmt::Debug {
    fn check(&self, item: &FallbackZoneThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneThresholdRequiredSegmentRule {
    pub message: String,
}

impl FallbackZoneThresholdRule for FallbackZoneThresholdRequiredSegmentRule {
    fn check(&self, item: &FallbackZoneThreshold) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneThresholdCodePrefixRule {
    pub prefix: String,
}

impl FallbackZoneThresholdRule for FallbackZoneThresholdCodePrefixRule {
    fn check(&self, item: &FallbackZoneThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneThresholdSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackZoneThresholdRule for FallbackZoneThresholdSequenceRangeRule {
    fn check(&self, item: &FallbackZoneThreshold) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback zone threshold.
#[derive(Debug, Clone)]
pub struct FallbackZoneThresholdValidator {
    pub rules: Vec<Rc<dyn FallbackZoneThresholdRule>>,
}

impl FallbackZoneThresholdValidator {
    pub fn validate(&self, item: &FallbackZoneThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackZoneThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackZoneThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_zone_threshold_validator() -> FallbackZoneThresholdValidator {
    FallbackZoneThresholdValidator {
        rules: vec![
            Rc::new(FallbackZoneThresholdRequiredSegmentRule {
                message: "fallback zone threshold needs an entry".to_string(),
            }) as Rc<dyn FallbackZoneThresholdRule>,
            Rc::new(FallbackZoneThresholdCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FallbackZoneThresholdSequenceRangeRule { min: 80, max: 1000 }),
        ],
    }
}

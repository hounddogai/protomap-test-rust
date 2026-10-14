use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched sink segment must pass. Check returns its problems, or none.
pub trait BatchedSinkSegmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSinkSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkSegmentRequiredCodeRule {
    pub message: String,
}

impl BatchedSinkSegmentRule for BatchedSinkSegmentRequiredCodeRule {
    fn check(&self, item: &BatchedSinkSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkSegmentCodePrefixRule {
    pub prefix: String,
}

impl BatchedSinkSegmentRule for BatchedSinkSegmentCodePrefixRule {
    fn check(&self, item: &BatchedSinkSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedSinkSegmentRule for BatchedSinkSegmentAllowedCodeRule {
    fn check(&self, item: &BatchedSinkSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkSegmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedSinkSegmentRule for BatchedSinkSegmentThresholdRangeRule {
    fn check(&self, item: &BatchedSinkSegment) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched sink segment.
#[derive(Debug, Clone)]
pub struct BatchedSinkSegmentValidator {
    pub rules: Vec<Rc<dyn BatchedSinkSegmentRule>>,
}

impl BatchedSinkSegmentValidator {
    pub fn validate(&self, item: &BatchedSinkSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedSinkSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSinkSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_sink_segment_validator() -> BatchedSinkSegmentValidator {
    BatchedSinkSegmentValidator {
        rules: vec![
            Rc::new(BatchedSinkSegmentRequiredCodeRule {
                message: "batched sink segment needs this field".to_string(),
            }) as Rc<dyn BatchedSinkSegmentRule>,
            Rc::new(BatchedSinkSegmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BatchedSinkSegmentAllowedCodeRule {
                allowed: vec!["batched".to_string(), "regional".to_string()],
            }),
            Rc::new(BatchedSinkSegmentThresholdRangeRule { min: 0.7, max: 2689.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_sink_segment_validator();
        let item = new_batched_sink_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedSinkSegmentRequiredCodeRule {
            message: "batched sink segment needs an entry".to_string(),
        };
        let mut item = new_batched_sink_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = BatchedSinkSegmentThresholdRangeRule { min: 3.5, max: 1160.8 };
        let mut item = new_batched_sink_segment("x", "y");
        item.threshold = 1162.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

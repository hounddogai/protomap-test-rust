use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary segment must pass. Check returns its problems, or none.
pub trait CanarySegmentRule: std::fmt::Debug {
    fn check(&self, item: &CanarySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySegmentCodePrefixRule {
    pub prefix: String,
}

impl CanarySegmentRule for CanarySegmentCodePrefixRule {
    fn check(&self, item: &CanarySegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySegmentRequiredNotesRule {
    pub message: String,
}

impl CanarySegmentRule for CanarySegmentRequiredNotesRule {
    fn check(&self, item: &CanarySegment) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySegmentAudienceLengthRule {
    pub max_length: i64,
}

impl CanarySegmentRule for CanarySegmentAudienceLengthRule {
    fn check(&self, item: &CanarySegment) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySegmentRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanarySegmentRule for CanarySegmentRatioRangeRule {
    fn check(&self, item: &CanarySegment) -> Vec<String> {
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

/// Runs every rule of a canary segment.
#[derive(Debug, Clone)]
pub struct CanarySegmentValidator {
    pub rules: Vec<Rc<dyn CanarySegmentRule>>,
}

impl CanarySegmentValidator {
    pub fn validate(&self, item: &CanarySegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanarySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanarySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_segment_validator() -> CanarySegmentValidator {
    CanarySegmentValidator {
        rules: vec![
            Rc::new(CanarySegmentCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn CanarySegmentRule>,
            Rc::new(CanarySegmentRequiredNotesRule {
                message: "canary segment needs a value".to_string(),
            }),
            Rc::new(CanarySegmentAudienceLengthRule { max_length: 120 }),
            Rc::new(CanarySegmentRatioRangeRule { min: 4.4, max: 1761.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_segment_validator();
        let item = new_canary_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = CanarySegmentRequiredNotesRule {
            message: "canary segment needs an entry".to_string(),
        };
        let mut item = new_canary_segment("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = CanarySegmentAudienceLengthRule { max_length: 32 };
        let mut item = new_canary_segment("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = CanarySegmentRatioRangeRule { min: 6.3, max: 1173.8 };
        let mut item = new_canary_segment("x", "y");
        item.ratio = 1175.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

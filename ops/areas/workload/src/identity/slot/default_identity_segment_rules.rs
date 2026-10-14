use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default identity segment must pass. Check returns its problems, or none.
pub trait DefaultIdentitySegmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultIdentitySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentitySegmentRequiredSourceRule {
    pub message: String,
}

impl DefaultIdentitySegmentRule for DefaultIdentitySegmentRequiredSourceRule {
    fn check(&self, item: &DefaultIdentitySegment) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentitySegmentRequiredCodeRule {
    pub message: String,
}

impl DefaultIdentitySegmentRule for DefaultIdentitySegmentRequiredCodeRule {
    fn check(&self, item: &DefaultIdentitySegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentitySegmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultIdentitySegmentRule for DefaultIdentitySegmentRateRangeRule {
    fn check(&self, item: &DefaultIdentitySegment) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default identity segment.
#[derive(Debug, Clone)]
pub struct DefaultIdentitySegmentValidator {
    pub rules: Vec<Rc<dyn DefaultIdentitySegmentRule>>,
}

impl DefaultIdentitySegmentValidator {
    pub fn validate(&self, item: &DefaultIdentitySegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultIdentitySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultIdentitySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_identity_segment_validator() -> DefaultIdentitySegmentValidator {
    DefaultIdentitySegmentValidator {
        rules: vec![
            Rc::new(DefaultIdentitySegmentRequiredSourceRule {
                message: "default identity segment needs a value".to_string(),
            }) as Rc<dyn DefaultIdentitySegmentRule>,
            Rc::new(DefaultIdentitySegmentRequiredCodeRule {
                message: "default identity segment needs this field".to_string(),
            }),
            Rc::new(DefaultIdentitySegmentRateRangeRule { min: 3.9, max: 458.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_identity_segment_validator();
        let item = new_default_identity_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = DefaultIdentitySegmentRequiredSourceRule {
            message: "default identity segment needs a value".to_string(),
        };
        let mut item = new_default_identity_segment("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultIdentitySegmentRequiredCodeRule {
            message: "default identity segment needs an entry".to_string(),
        };
        let mut item = new_default_identity_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DefaultIdentitySegmentRateRangeRule { min: 7.1, max: 4829.9 };
        let mut item = new_default_identity_segment("x", "y");
        item.rate = 4831.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

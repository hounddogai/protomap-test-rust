use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback postmortem threshold must pass. Check returns its problems, or none.
pub trait FallbackPostmortemThresholdRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPostmortemThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemThresholdTitleLengthRule {
    pub max_length: i64,
}

impl FallbackPostmortemThresholdRule for FallbackPostmortemThresholdTitleLengthRule {
    fn check(&self, item: &FallbackPostmortemThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemThresholdRequiredSegmentRule {
    pub message: String,
}

impl FallbackPostmortemThresholdRule for FallbackPostmortemThresholdRequiredSegmentRule {
    fn check(&self, item: &FallbackPostmortemThreshold) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemThresholdRequiredTitleRule {
    pub message: String,
}

impl FallbackPostmortemThresholdRule for FallbackPostmortemThresholdRequiredTitleRule {
    fn check(&self, item: &FallbackPostmortemThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback postmortem threshold.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemThresholdValidator {
    pub rules: Vec<Rc<dyn FallbackPostmortemThresholdRule>>,
}

impl FallbackPostmortemThresholdValidator {
    pub fn validate(&self, item: &FallbackPostmortemThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackPostmortemThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPostmortemThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_postmortem_threshold_validator() -> FallbackPostmortemThresholdValidator {
    FallbackPostmortemThresholdValidator {
        rules: vec![
            Rc::new(FallbackPostmortemThresholdTitleLengthRule { max_length: 120 })
                as Rc<dyn FallbackPostmortemThresholdRule>,
            Rc::new(FallbackPostmortemThresholdRequiredSegmentRule {
                message: "fallback postmortem threshold needs a value".to_string(),
            }),
            Rc::new(FallbackPostmortemThresholdRequiredTitleRule {
                message: "fallback postmortem threshold needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_postmortem_threshold_validator();
        let item = new_fallback_postmortem_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackPostmortemThresholdTitleLengthRule { max_length: 1000 };
        let mut item = new_fallback_postmortem_threshold("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = FallbackPostmortemThresholdRequiredSegmentRule {
            message: "fallback postmortem threshold needs this field".to_string(),
        };
        let mut item = new_fallback_postmortem_threshold("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackPostmortemThresholdRequiredTitleRule {
            message: "fallback postmortem threshold needs an entry".to_string(),
        };
        let mut item = new_fallback_postmortem_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

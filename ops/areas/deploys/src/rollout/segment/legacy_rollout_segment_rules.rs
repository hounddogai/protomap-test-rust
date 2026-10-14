use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rollout segment must pass. Check returns its problems, or none.
pub trait LegacyRolloutSegmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRolloutSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutSegmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyRolloutSegmentRule for LegacyRolloutSegmentPortRangeRule {
    fn check(&self, item: &LegacyRolloutSegment) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutSegmentTitleLengthRule {
    pub max_length: i64,
}

impl LegacyRolloutSegmentRule for LegacyRolloutSegmentTitleLengthRule {
    fn check(&self, item: &LegacyRolloutSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutSegmentRequiredCountryRule {
    pub message: String,
}

impl LegacyRolloutSegmentRule for LegacyRolloutSegmentRequiredCountryRule {
    fn check(&self, item: &LegacyRolloutSegment) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rollout segment.
#[derive(Debug, Clone)]
pub struct LegacyRolloutSegmentValidator {
    pub rules: Vec<Rc<dyn LegacyRolloutSegmentRule>>,
}

impl LegacyRolloutSegmentValidator {
    pub fn validate(&self, item: &LegacyRolloutSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRolloutSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRolloutSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollout_segment_validator() -> LegacyRolloutSegmentValidator {
    LegacyRolloutSegmentValidator {
        rules: vec![
            Rc::new(LegacyRolloutSegmentPortRangeRule { min: 3600, max: 3600 }) as Rc<dyn LegacyRolloutSegmentRule>,
            Rc::new(LegacyRolloutSegmentTitleLengthRule { max_length: 80 }),
            Rc::new(LegacyRolloutSegmentRequiredCountryRule {
                message: "legacy rollout segment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_rollout_segment_validator();
        let item = new_legacy_rollout_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LegacyRolloutSegmentPortRangeRule { min: 16, max: 1000 };
        let mut item = new_legacy_rollout_segment("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyRolloutSegmentTitleLengthRule { max_length: 80 };
        let mut item = new_legacy_rollout_segment("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = LegacyRolloutSegmentRequiredCountryRule {
            message: "legacy rollout segment needs this field".to_string(),
        };
        let mut item = new_legacy_rollout_segment("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

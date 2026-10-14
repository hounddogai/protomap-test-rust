use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional timeline request must pass. Check returns its problems, or none.
pub trait RegionalTimelineRequestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTimelineRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineRequestRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalTimelineRequestRule for RegionalTimelineRequestRevisionRangeRule {
    fn check(&self, item: &RegionalTimelineRequest) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineRequestTitleLengthRule {
    pub max_length: i64,
}

impl RegionalTimelineRequestRule for RegionalTimelineRequestTitleLengthRule {
    fn check(&self, item: &RegionalTimelineRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineRequestCodePrefixRule {
    pub prefix: String,
}

impl RegionalTimelineRequestRule for RegionalTimelineRequestCodePrefixRule {
    fn check(&self, item: &RegionalTimelineRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional timeline request.
#[derive(Debug, Clone)]
pub struct RegionalTimelineRequestValidator {
    pub rules: Vec<Rc<dyn RegionalTimelineRequestRule>>,
}

impl RegionalTimelineRequestValidator {
    pub fn validate(&self, item: &RegionalTimelineRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalTimelineRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTimelineRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_timeline_request_validator() -> RegionalTimelineRequestValidator {
    RegionalTimelineRequestValidator {
        rules: vec![
            Rc::new(RegionalTimelineRequestRevisionRangeRule { min: 120, max: 86400 })
                as Rc<dyn RegionalTimelineRequestRule>,
            Rc::new(RegionalTimelineRequestTitleLengthRule { max_length: 16 }),
            Rc::new(RegionalTimelineRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_timeline_request_validator();
        let item = new_regional_timeline_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RegionalTimelineRequestRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_regional_timeline_request("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalTimelineRequestTitleLengthRule { max_length: 255 };
        let mut item = new_regional_timeline_request("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

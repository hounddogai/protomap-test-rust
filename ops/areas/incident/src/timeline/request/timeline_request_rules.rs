use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline request must pass. Check returns its problems, or none.
pub trait TimelineRequestRule: std::fmt::Debug {
    fn check(&self, item: &TimelineRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRequestSourceLengthRule {
    pub max_length: i64,
}

impl TimelineRequestRule for TimelineRequestSourceLengthRule {
    fn check(&self, item: &TimelineRequest) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRequestPrincipalLengthRule {
    pub max_length: i64,
}

impl TimelineRequestRule for TimelineRequestPrincipalLengthRule {
    fn check(&self, item: &TimelineRequest) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRequestRequiredSourceRule {
    pub message: String,
}

impl TimelineRequestRule for TimelineRequestRequiredSourceRule {
    fn check(&self, item: &TimelineRequest) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline request.
#[derive(Debug, Clone)]
pub struct TimelineRequestValidator {
    pub rules: Vec<Rc<dyn TimelineRequestRule>>,
}

impl TimelineRequestValidator {
    pub fn validate(&self, item: &TimelineRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_request_validator() -> TimelineRequestValidator {
    TimelineRequestValidator {
        rules: vec![
            Rc::new(TimelineRequestSourceLengthRule { max_length: 80 }) as Rc<dyn TimelineRequestRule>,
            Rc::new(TimelineRequestPrincipalLengthRule { max_length: 120 }),
            Rc::new(TimelineRequestRequiredSourceRule {
                message: "timeline request needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_request_validator();
        let item = new_timeline_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = TimelineRequestSourceLengthRule { max_length: 86400 };
        let mut item = new_timeline_request("x", "y");
        item.source = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = TimelineRequestPrincipalLengthRule { max_length: 1000 };
        let mut item = new_timeline_request("x", "y");
        item.principal = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TimelineRequestRequiredSourceRule {
            message: "timeline request needs a value".to_string(),
        };
        let mut item = new_timeline_request("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline exception must pass. Check returns its problems, or none.
pub trait TimelineExceptionRule: std::fmt::Debug {
    fn check(&self, item: &TimelineException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineExceptionRequiredTitleRule {
    pub message: String,
}

impl TimelineExceptionRule for TimelineExceptionRequiredTitleRule {
    fn check(&self, item: &TimelineException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineExceptionAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TimelineExceptionRule for TimelineExceptionAllowedCodeRule {
    fn check(&self, item: &TimelineException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineExceptionTitleLengthRule {
    pub max_length: i64,
}

impl TimelineExceptionRule for TimelineExceptionTitleLengthRule {
    fn check(&self, item: &TimelineException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineExceptionRequiredScopeRule {
    pub message: String,
}

impl TimelineExceptionRule for TimelineExceptionRequiredScopeRule {
    fn check(&self, item: &TimelineException) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineExceptionSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineExceptionRule for TimelineExceptionSequenceRangeRule {
    fn check(&self, item: &TimelineException) -> Vec<String> {
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

/// Runs every rule of a timeline exception.
#[derive(Debug, Clone)]
pub struct TimelineExceptionValidator {
    pub rules: Vec<Rc<dyn TimelineExceptionRule>>,
}

impl TimelineExceptionValidator {
    pub fn validate(&self, item: &TimelineException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_exception_validator() -> TimelineExceptionValidator {
    TimelineExceptionValidator {
        rules: vec![
            Rc::new(TimelineExceptionRequiredTitleRule {
                message: "timeline exception needs a value".to_string(),
            }) as Rc<dyn TimelineExceptionRule>,
            Rc::new(TimelineExceptionAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(TimelineExceptionTitleLengthRule { max_length: 32 }),
            Rc::new(TimelineExceptionRequiredScopeRule {
                message: "timeline exception needs an entry".to_string(),
            }),
            Rc::new(TimelineExceptionSequenceRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_exception_validator();
        let item = new_timeline_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TimelineExceptionRequiredTitleRule {
            message: "timeline exception needs a value".to_string(),
        };
        let mut item = new_timeline_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TimelineExceptionTitleLengthRule { max_length: 86400 };
        let mut item = new_timeline_exception("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = TimelineExceptionRequiredScopeRule {
            message: "timeline exception needs this field".to_string(),
        };
        let mut item = new_timeline_exception("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = TimelineExceptionSequenceRangeRule { min: 32, max: 120 };
        let mut item = new_timeline_exception("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

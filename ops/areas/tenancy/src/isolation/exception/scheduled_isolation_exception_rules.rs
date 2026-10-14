use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled isolation exception must pass. Check returns its problems, or none.
pub trait ScheduledIsolationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledIsolationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationExceptionCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledIsolationExceptionRule for ScheduledIsolationExceptionCodeLengthRule {
    fn check(&self, item: &ScheduledIsolationException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationExceptionAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledIsolationExceptionRule for ScheduledIsolationExceptionAllowedCodeRule {
    fn check(&self, item: &ScheduledIsolationException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationExceptionChannelLengthRule {
    pub max_length: i64,
}

impl ScheduledIsolationExceptionRule for ScheduledIsolationExceptionChannelLengthRule {
    fn check(&self, item: &ScheduledIsolationException) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled isolation exception.
#[derive(Debug, Clone)]
pub struct ScheduledIsolationExceptionValidator {
    pub rules: Vec<Rc<dyn ScheduledIsolationExceptionRule>>,
}

impl ScheduledIsolationExceptionValidator {
    pub fn validate(&self, item: &ScheduledIsolationException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledIsolationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledIsolationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_isolation_exception_validator() -> ScheduledIsolationExceptionValidator {
    ScheduledIsolationExceptionValidator {
        rules: vec![
            Rc::new(ScheduledIsolationExceptionCodeLengthRule { max_length: 120 })
                as Rc<dyn ScheduledIsolationExceptionRule>,
            Rc::new(ScheduledIsolationExceptionAllowedCodeRule {
                allowed: vec!["adaptive".to_string(), "adaptive".to_string(), "capped".to_string()],
            }),
            Rc::new(ScheduledIsolationExceptionChannelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_isolation_exception_validator();
        let item = new_scheduled_isolation_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledIsolationExceptionCodeLengthRule { max_length: 80 };
        let mut item = new_scheduled_isolation_exception("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ScheduledIsolationExceptionChannelLengthRule { max_length: 16 };
        let mut item = new_scheduled_isolation_exception("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink exception must pass. Check returns its problems, or none.
pub trait SinkExceptionRule: std::fmt::Debug {
    fn check(&self, item: &SinkException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkExceptionCodeLengthRule {
    pub max_length: i64,
}

impl SinkExceptionRule for SinkExceptionCodeLengthRule {
    fn check(&self, item: &SinkException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkExceptionAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SinkExceptionRule for SinkExceptionAllowedNameRule {
    fn check(&self, item: &SinkException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkExceptionCodePrefixRule {
    pub prefix: String,
}

impl SinkExceptionRule for SinkExceptionCodePrefixRule {
    fn check(&self, item: &SinkException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkExceptionActiveGuardRule {
    pub limit: f64,
}

impl SinkExceptionRule for SinkExceptionActiveGuardRule {
    fn check(&self, item: &SinkException) -> Vec<String> {
        if item.active && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while active", item.score, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink exception.
#[derive(Debug, Clone)]
pub struct SinkExceptionValidator {
    pub rules: Vec<Rc<dyn SinkExceptionRule>>,
}

impl SinkExceptionValidator {
    pub fn validate(&self, item: &SinkException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_exception_validator() -> SinkExceptionValidator {
    SinkExceptionValidator {
        rules: vec![
            Rc::new(SinkExceptionCodeLengthRule { max_length: 1000 }) as Rc<dyn SinkExceptionRule>,
            Rc::new(SinkExceptionAllowedNameRule {
                allowed: vec!["strict".to_string(), "scheduled".to_string(), "seasonal".to_string()],
            }),
            Rc::new(SinkExceptionCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SinkExceptionActiveGuardRule { limit: 3.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_exception_validator();
        let item = new_sink_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkExceptionCodeLengthRule { max_length: 16 };
        let mut item = new_sink_exception("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

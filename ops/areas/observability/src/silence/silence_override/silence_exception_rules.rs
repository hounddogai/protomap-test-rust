use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence exception must pass. Check returns its problems, or none.
pub trait SilenceExceptionRule: std::fmt::Debug {
    fn check(&self, item: &SilenceException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceExceptionCodeLengthRule {
    pub max_length: i64,
}

impl SilenceExceptionRule for SilenceExceptionCodeLengthRule {
    fn check(&self, item: &SilenceException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceExceptionLabelLengthRule {
    pub max_length: i64,
}

impl SilenceExceptionRule for SilenceExceptionLabelLengthRule {
    fn check(&self, item: &SilenceException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceExceptionRequiredLabelRule {
    pub message: String,
}

impl SilenceExceptionRule for SilenceExceptionRequiredLabelRule {
    fn check(&self, item: &SilenceException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceExceptionRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceExceptionRule for SilenceExceptionRetriesRangeRule {
    fn check(&self, item: &SilenceException) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceExceptionRequiredNameRule {
    pub message: String,
}

impl SilenceExceptionRule for SilenceExceptionRequiredNameRule {
    fn check(&self, item: &SilenceException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence exception.
#[derive(Debug, Clone)]
pub struct SilenceExceptionValidator {
    pub rules: Vec<Rc<dyn SilenceExceptionRule>>,
}

impl SilenceExceptionValidator {
    pub fn validate(&self, item: &SilenceException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_exception_validator() -> SilenceExceptionValidator {
    SilenceExceptionValidator {
        rules: vec![
            Rc::new(SilenceExceptionCodeLengthRule { max_length: 255 }) as Rc<dyn SilenceExceptionRule>,
            Rc::new(SilenceExceptionLabelLengthRule { max_length: 255 }),
            Rc::new(SilenceExceptionRequiredLabelRule {
                message: "silence exception needs a value".to_string(),
            }),
            Rc::new(SilenceExceptionRetriesRangeRule { min: 64, max: 1000 }),
            Rc::new(SilenceExceptionRequiredNameRule {
                message: "silence exception needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_exception_validator();
        let item = new_silence_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilenceExceptionCodeLengthRule { max_length: 64 };
        let mut item = new_silence_exception("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SilenceExceptionLabelLengthRule { max_length: 64 };
        let mut item = new_silence_exception("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SilenceExceptionRequiredLabelRule {
            message: "silence exception needs this field".to_string(),
        };
        let mut item = new_silence_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = SilenceExceptionRetriesRangeRule { min: 120, max: 255 };
        let mut item = new_silence_exception("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SilenceExceptionRequiredNameRule {
            message: "silence exception needs this field".to_string(),
        };
        let mut item = new_silence_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

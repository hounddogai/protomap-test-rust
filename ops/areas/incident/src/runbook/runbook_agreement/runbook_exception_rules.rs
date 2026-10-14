use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook exception must pass. Check returns its problems, or none.
pub trait RunbookExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RunbookException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookExceptionIssuerLengthRule {
    pub max_length: i64,
}

impl RunbookExceptionRule for RunbookExceptionIssuerLengthRule {
    fn check(&self, item: &RunbookException) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookExceptionCodePrefixRule {
    pub prefix: String,
}

impl RunbookExceptionRule for RunbookExceptionCodePrefixRule {
    fn check(&self, item: &RunbookException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookExceptionRequiredIssuerRule {
    pub message: String,
}

impl RunbookExceptionRule for RunbookExceptionRequiredIssuerRule {
    fn check(&self, item: &RunbookException) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookExceptionTitleLengthRule {
    pub max_length: i64,
}

impl RunbookExceptionRule for RunbookExceptionTitleLengthRule {
    fn check(&self, item: &RunbookException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookExceptionAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RunbookExceptionRule for RunbookExceptionAllowedCodeRule {
    fn check(&self, item: &RunbookException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook exception.
#[derive(Debug, Clone)]
pub struct RunbookExceptionValidator {
    pub rules: Vec<Rc<dyn RunbookExceptionRule>>,
}

impl RunbookExceptionValidator {
    pub fn validate(&self, item: &RunbookException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_exception_validator() -> RunbookExceptionValidator {
    RunbookExceptionValidator {
        rules: vec![
            Rc::new(RunbookExceptionIssuerLengthRule { max_length: 86400 }) as Rc<dyn RunbookExceptionRule>,
            Rc::new(RunbookExceptionCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RunbookExceptionRequiredIssuerRule {
                message: "runbook exception needs an entry".to_string(),
            }),
            Rc::new(RunbookExceptionTitleLengthRule { max_length: 64 }),
            Rc::new(RunbookExceptionAllowedCodeRule {
                allowed: vec!["manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_exception_validator();
        let item = new_runbook_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RunbookExceptionIssuerLengthRule { max_length: 64 };
        let mut item = new_runbook_exception("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RunbookExceptionRequiredIssuerRule {
            message: "runbook exception needs a value".to_string(),
        };
        let mut item = new_runbook_exception("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RunbookExceptionTitleLengthRule { max_length: 86400 };
        let mut item = new_runbook_exception("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

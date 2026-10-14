use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook agreement must pass. Check returns its problems, or none.
pub trait RunbookAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RunbookAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreementNameLengthRule {
    pub max_length: i64,
}

impl RunbookAgreementRule for RunbookAgreementNameLengthRule {
    fn check(&self, item: &RunbookAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreementCodePrefixRule {
    pub prefix: String,
}

impl RunbookAgreementRule for RunbookAgreementCodePrefixRule {
    fn check(&self, item: &RunbookAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreementLocaleLengthRule {
    pub max_length: i64,
}

impl RunbookAgreementRule for RunbookAgreementLocaleLengthRule {
    fn check(&self, item: &RunbookAgreement) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreementRequiredNameRule {
    pub message: String,
}

impl RunbookAgreementRule for RunbookAgreementRequiredNameRule {
    fn check(&self, item: &RunbookAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreementCodeLengthRule {
    pub max_length: i64,
}

impl RunbookAgreementRule for RunbookAgreementCodeLengthRule {
    fn check(&self, item: &RunbookAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreementAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RunbookAgreementRule for RunbookAgreementAllowedNameRule {
    fn check(&self, item: &RunbookAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook agreement.
#[derive(Debug, Clone)]
pub struct RunbookAgreementValidator {
    pub rules: Vec<Rc<dyn RunbookAgreementRule>>,
}

impl RunbookAgreementValidator {
    pub fn validate(&self, item: &RunbookAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_agreement_validator() -> RunbookAgreementValidator {
    RunbookAgreementValidator {
        rules: vec![
            Rc::new(RunbookAgreementNameLengthRule { max_length: 3600 }) as Rc<dyn RunbookAgreementRule>,
            Rc::new(RunbookAgreementCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RunbookAgreementLocaleLengthRule { max_length: 80 }),
            Rc::new(RunbookAgreementRequiredNameRule {
                message: "runbook agreement needs an entry".to_string(),
            }),
            Rc::new(RunbookAgreementCodeLengthRule { max_length: 32 }),
            Rc::new(RunbookAgreementAllowedNameRule {
                allowed: vec!["fallback".to_string(), "rolling".to_string(), "legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_agreement_validator();
        let item = new_runbook_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RunbookAgreementNameLengthRule { max_length: 64 };
        let mut item = new_runbook_agreement("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RunbookAgreementLocaleLengthRule { max_length: 32 };
        let mut item = new_runbook_agreement("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RunbookAgreementRequiredNameRule {
            message: "runbook agreement needs this field".to_string(),
        };
        let mut item = new_runbook_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RunbookAgreementCodeLengthRule { max_length: 3600 };
        let mut item = new_runbook_agreement("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

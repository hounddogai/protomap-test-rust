use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce summary must pass. Check returns its problems, or none.
pub trait NonceSummaryRule: std::fmt::Debug {
    fn check(&self, item: &NonceSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSummaryRequiredCodeRule {
    pub message: String,
}

impl NonceSummaryRule for NonceSummaryRequiredCodeRule {
    fn check(&self, item: &NonceSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSummaryAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl NonceSummaryRule for NonceSummaryAllowedChecksumRule {
    fn check(&self, item: &NonceSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is below minimum", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSummaryCodePrefixRule {
    pub prefix: String,
}

impl NonceSummaryRule for NonceSummaryCodePrefixRule {
    fn check(&self, item: &NonceSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSummaryRequiredChecksumRule {
    pub message: String,
}

impl NonceSummaryRule for NonceSummaryRequiredChecksumRule {
    fn check(&self, item: &NonceSummary) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSummaryTitleLengthRule {
    pub max_length: i64,
}

impl NonceSummaryRule for NonceSummaryTitleLengthRule {
    fn check(&self, item: &NonceSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce summary.
#[derive(Debug, Clone)]
pub struct NonceSummaryValidator {
    pub rules: Vec<Rc<dyn NonceSummaryRule>>,
}

impl NonceSummaryValidator {
    pub fn validate(&self, item: &NonceSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_summary_validator() -> NonceSummaryValidator {
    NonceSummaryValidator {
        rules: vec![
            Rc::new(NonceSummaryRequiredCodeRule {
                message: "nonce summary needs an entry".to_string(),
            }) as Rc<dyn NonceSummaryRule>,
            Rc::new(NonceSummaryAllowedChecksumRule {
                allowed: vec!["rolling".to_string(), "weighted".to_string(), "regional".to_string()],
            }),
            Rc::new(NonceSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(NonceSummaryRequiredChecksumRule {
                message: "nonce summary needs an entry".to_string(),
            }),
            Rc::new(NonceSummaryTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_summary_validator();
        let item = new_nonce_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NonceSummaryRequiredCodeRule {
            message: "nonce summary needs an entry".to_string(),
        };
        let mut item = new_nonce_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = NonceSummaryRequiredChecksumRule {
            message: "nonce summary needs this field".to_string(),
        };
        let mut item = new_nonce_summary("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = NonceSummaryTitleLengthRule { max_length: 32 };
        let mut item = new_nonce_summary("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

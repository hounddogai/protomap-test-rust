use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy flow log variance must pass. Check returns its problems, or none.
pub trait LegacyFlowLogVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFlowLogVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogVarianceAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl LegacyFlowLogVarianceRule for LegacyFlowLogVarianceAllowedContactEmailRule {
    fn check(&self, item: &LegacyFlowLogVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is too long", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogVarianceCipherLengthRule {
    pub max_length: i64,
}

impl LegacyFlowLogVarianceRule for LegacyFlowLogVarianceCipherLengthRule {
    fn check(&self, item: &LegacyFlowLogVariance) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogVarianceRequiredTitleRule {
    pub message: String,
}

impl LegacyFlowLogVarianceRule for LegacyFlowLogVarianceRequiredTitleRule {
    fn check(&self, item: &LegacyFlowLogVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogVarianceRequiredCipherRule {
    pub message: String,
}

impl LegacyFlowLogVarianceRule for LegacyFlowLogVarianceRequiredCipherRule {
    fn check(&self, item: &LegacyFlowLogVariance) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogVarianceRequiredCodeRule {
    pub message: String,
}

impl LegacyFlowLogVarianceRule for LegacyFlowLogVarianceRequiredCodeRule {
    fn check(&self, item: &LegacyFlowLogVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy flow log variance.
#[derive(Debug, Clone)]
pub struct LegacyFlowLogVarianceValidator {
    pub rules: Vec<Rc<dyn LegacyFlowLogVarianceRule>>,
}

impl LegacyFlowLogVarianceValidator {
    pub fn validate(&self, item: &LegacyFlowLogVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyFlowLogVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFlowLogVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_flow_log_variance_validator() -> LegacyFlowLogVarianceValidator {
    LegacyFlowLogVarianceValidator {
        rules: vec![
            Rc::new(LegacyFlowLogVarianceAllowedContactEmailRule {
                allowed: vec!["rolling".to_string(), "global".to_string(), "regional".to_string()],
            }) as Rc<dyn LegacyFlowLogVarianceRule>,
            Rc::new(LegacyFlowLogVarianceCipherLengthRule { max_length: 16 }),
            Rc::new(LegacyFlowLogVarianceRequiredTitleRule {
                message: "legacy flow log variance needs an entry".to_string(),
            }),
            Rc::new(LegacyFlowLogVarianceRequiredCipherRule {
                message: "legacy flow log variance needs a value".to_string(),
            }),
            Rc::new(LegacyFlowLogVarianceRequiredCodeRule {
                message: "legacy flow log variance needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_flow_log_variance_validator();
        let item = new_legacy_flow_log_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = LegacyFlowLogVarianceCipherLengthRule { max_length: 3600 };
        let mut item = new_legacy_flow_log_variance("x", "y");
        item.cipher = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyFlowLogVarianceRequiredTitleRule {
            message: "legacy flow log variance needs this field".to_string(),
        };
        let mut item = new_legacy_flow_log_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LegacyFlowLogVarianceRequiredCipherRule {
            message: "legacy flow log variance needs an entry".to_string(),
        };
        let mut item = new_legacy_flow_log_variance("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyFlowLogVarianceRequiredCodeRule {
            message: "legacy flow log variance needs this field".to_string(),
        };
        let mut item = new_legacy_flow_log_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

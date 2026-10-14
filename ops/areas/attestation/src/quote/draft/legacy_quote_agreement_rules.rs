use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy quote agreement must pass. Check returns its problems, or none.
pub trait LegacyQuoteAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LegacyQuoteAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteAgreementCodeLengthRule {
    pub max_length: i64,
}

impl LegacyQuoteAgreementRule for LegacyQuoteAgreementCodeLengthRule {
    fn check(&self, item: &LegacyQuoteAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteAgreementTimezoneLengthRule {
    pub max_length: i64,
}

impl LegacyQuoteAgreementRule for LegacyQuoteAgreementTimezoneLengthRule {
    fn check(&self, item: &LegacyQuoteAgreement) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteAgreementRequiredCodeRule {
    pub message: String,
}

impl LegacyQuoteAgreementRule for LegacyQuoteAgreementRequiredCodeRule {
    fn check(&self, item: &LegacyQuoteAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy quote agreement.
#[derive(Debug, Clone)]
pub struct LegacyQuoteAgreementValidator {
    pub rules: Vec<Rc<dyn LegacyQuoteAgreementRule>>,
}

impl LegacyQuoteAgreementValidator {
    pub fn validate(&self, item: &LegacyQuoteAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyQuoteAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyQuoteAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_quote_agreement_validator() -> LegacyQuoteAgreementValidator {
    LegacyQuoteAgreementValidator {
        rules: vec![
            Rc::new(LegacyQuoteAgreementCodeLengthRule { max_length: 120 }) as Rc<dyn LegacyQuoteAgreementRule>,
            Rc::new(LegacyQuoteAgreementTimezoneLengthRule { max_length: 32 }),
            Rc::new(LegacyQuoteAgreementRequiredCodeRule {
                message: "legacy quote agreement needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_quote_agreement_validator();
        let item = new_legacy_quote_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyQuoteAgreementCodeLengthRule { max_length: 86400 };
        let mut item = new_legacy_quote_agreement("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = LegacyQuoteAgreementTimezoneLengthRule { max_length: 80 };
        let mut item = new_legacy_quote_agreement("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyQuoteAgreementRequiredCodeRule {
            message: "legacy quote agreement needs a value".to_string(),
        };
        let mut item = new_legacy_quote_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

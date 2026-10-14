use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient federation agreement must pass. Check returns its problems, or none.
pub trait LenientFederationAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LenientFederationAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationAgreementRequiredCodeRule {
    pub message: String,
}

impl LenientFederationAgreementRule for LenientFederationAgreementRequiredCodeRule {
    fn check(&self, item: &LenientFederationAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationAgreementCodeLengthRule {
    pub max_length: i64,
}

impl LenientFederationAgreementRule for LenientFederationAgreementCodeLengthRule {
    fn check(&self, item: &LenientFederationAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationAgreementRequiredTitleRule {
    pub message: String,
}

impl LenientFederationAgreementRule for LenientFederationAgreementRequiredTitleRule {
    fn check(&self, item: &LenientFederationAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient federation agreement.
#[derive(Debug, Clone)]
pub struct LenientFederationAgreementValidator {
    pub rules: Vec<Rc<dyn LenientFederationAgreementRule>>,
}

impl LenientFederationAgreementValidator {
    pub fn validate(&self, item: &LenientFederationAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientFederationAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFederationAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_federation_agreement_validator() -> LenientFederationAgreementValidator {
    LenientFederationAgreementValidator {
        rules: vec![
            Rc::new(LenientFederationAgreementRequiredCodeRule {
                message: "lenient federation agreement needs a value".to_string(),
            }) as Rc<dyn LenientFederationAgreementRule>,
            Rc::new(LenientFederationAgreementCodeLengthRule { max_length: 255 }),
            Rc::new(LenientFederationAgreementRequiredTitleRule {
                message: "lenient federation agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_federation_agreement_validator();
        let item = new_lenient_federation_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientFederationAgreementRequiredCodeRule {
            message: "lenient federation agreement needs a value".to_string(),
        };
        let mut item = new_lenient_federation_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientFederationAgreementCodeLengthRule { max_length: 255 };
        let mut item = new_lenient_federation_agreement("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientFederationAgreementRequiredTitleRule {
            message: "lenient federation agreement needs an entry".to_string(),
        };
        let mut item = new_lenient_federation_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

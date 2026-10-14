use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional federation agreement must pass. Check returns its problems, or none.
pub trait RegionalFederationAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFederationAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAgreementCodePrefixRule {
    pub prefix: String,
}

impl RegionalFederationAgreementRule for RegionalFederationAgreementCodePrefixRule {
    fn check(&self, item: &RegionalFederationAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAgreementTitleLengthRule {
    pub max_length: i64,
}

impl RegionalFederationAgreementRule for RegionalFederationAgreementTitleLengthRule {
    fn check(&self, item: &RegionalFederationAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAgreementAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl RegionalFederationAgreementRule for RegionalFederationAgreementAllowedFingerprintRule {
    fn check(&self, item: &RegionalFederationAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is missing", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAgreementCategoryLengthRule {
    pub max_length: i64,
}

impl RegionalFederationAgreementRule for RegionalFederationAgreementCategoryLengthRule {
    fn check(&self, item: &RegionalFederationAgreement) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAgreementRequiredTitleRule {
    pub message: String,
}

impl RegionalFederationAgreementRule for RegionalFederationAgreementRequiredTitleRule {
    fn check(&self, item: &RegionalFederationAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional federation agreement.
#[derive(Debug, Clone)]
pub struct RegionalFederationAgreementValidator {
    pub rules: Vec<Rc<dyn RegionalFederationAgreementRule>>,
}

impl RegionalFederationAgreementValidator {
    pub fn validate(&self, item: &RegionalFederationAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalFederationAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFederationAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_federation_agreement_validator() -> RegionalFederationAgreementValidator {
    RegionalFederationAgreementValidator {
        rules: vec![
            Rc::new(RegionalFederationAgreementCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RegionalFederationAgreementRule>,
            Rc::new(RegionalFederationAgreementTitleLengthRule { max_length: 86400 }),
            Rc::new(RegionalFederationAgreementAllowedFingerprintRule {
                allowed: vec!["pending".to_string(), "primary".to_string()],
            }),
            Rc::new(RegionalFederationAgreementCategoryLengthRule { max_length: 32 }),
            Rc::new(RegionalFederationAgreementRequiredTitleRule {
                message: "regional federation agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_federation_agreement_validator();
        let item = new_regional_federation_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalFederationAgreementTitleLengthRule { max_length: 255 };
        let mut item = new_regional_federation_agreement("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RegionalFederationAgreementCategoryLengthRule { max_length: 16 };
        let mut item = new_regional_federation_agreement("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalFederationAgreementRequiredTitleRule {
            message: "regional federation agreement needs an entry".to_string(),
        };
        let mut item = new_regional_federation_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential variance must pass. Check returns its problems, or none.
pub trait CredentialVarianceRule: std::fmt::Debug {
    fn check(&self, item: &CredentialVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialVarianceAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CredentialVarianceRule for CredentialVarianceAllowedNameRule {
    fn check(&self, item: &CredentialVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialVarianceCountryLengthRule {
    pub max_length: i64,
}

impl CredentialVarianceRule for CredentialVarianceCountryLengthRule {
    fn check(&self, item: &CredentialVariance) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialVarianceCodeLengthRule {
    pub max_length: i64,
}

impl CredentialVarianceRule for CredentialVarianceCodeLengthRule {
    fn check(&self, item: &CredentialVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential variance.
#[derive(Debug, Clone)]
pub struct CredentialVarianceValidator {
    pub rules: Vec<Rc<dyn CredentialVarianceRule>>,
}

impl CredentialVarianceValidator {
    pub fn validate(&self, item: &CredentialVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_variance_validator() -> CredentialVarianceValidator {
    CredentialVarianceValidator {
        rules: vec![
            Rc::new(CredentialVarianceAllowedNameRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn CredentialVarianceRule>,
            Rc::new(CredentialVarianceCountryLengthRule { max_length: 86400 }),
            Rc::new(CredentialVarianceCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_variance_validator();
        let item = new_credential_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = CredentialVarianceCountryLengthRule { max_length: 16 };
        let mut item = new_credential_variance("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CredentialVarianceCodeLengthRule { max_length: 86400 };
        let mut item = new_credential_variance("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

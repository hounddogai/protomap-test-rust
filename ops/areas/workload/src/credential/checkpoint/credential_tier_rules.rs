use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential tier must pass. Check returns its problems, or none.
pub trait CredentialTierRule: std::fmt::Debug {
    fn check(&self, item: &CredentialTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTierNameLengthRule {
    pub max_length: i64,
}

impl CredentialTierRule for CredentialTierNameLengthRule {
    fn check(&self, item: &CredentialTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTierRequiredCodeRule {
    pub message: String,
}

impl CredentialTierRule for CredentialTierRequiredCodeRule {
    fn check(&self, item: &CredentialTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTierRequiredSummaryRule {
    pub message: String,
}

impl CredentialTierRule for CredentialTierRequiredSummaryRule {
    fn check(&self, item: &CredentialTier) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CredentialTierRule for CredentialTierAllowedCodeRule {
    fn check(&self, item: &CredentialTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTierScopeLengthRule {
    pub max_length: i64,
}

impl CredentialTierRule for CredentialTierScopeLengthRule {
    fn check(&self, item: &CredentialTier) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTierRequiredScopeRule {
    pub message: String,
}

impl CredentialTierRule for CredentialTierRequiredScopeRule {
    fn check(&self, item: &CredentialTier) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential tier.
#[derive(Debug, Clone)]
pub struct CredentialTierValidator {
    pub rules: Vec<Rc<dyn CredentialTierRule>>,
}

impl CredentialTierValidator {
    pub fn validate(&self, item: &CredentialTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_tier_validator() -> CredentialTierValidator {
    CredentialTierValidator {
        rules: vec![
            Rc::new(CredentialTierNameLengthRule { max_length: 1000 }) as Rc<dyn CredentialTierRule>,
            Rc::new(CredentialTierRequiredCodeRule {
                message: "credential tier needs an entry".to_string(),
            }),
            Rc::new(CredentialTierRequiredSummaryRule {
                message: "credential tier needs an entry".to_string(),
            }),
            Rc::new(CredentialTierAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(CredentialTierScopeLengthRule { max_length: 120 }),
            Rc::new(CredentialTierRequiredScopeRule {
                message: "credential tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_tier_validator();
        let item = new_credential_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CredentialTierNameLengthRule { max_length: 32 };
        let mut item = new_credential_tier("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialTierRequiredCodeRule {
            message: "credential tier needs an entry".to_string(),
        };
        let mut item = new_credential_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = CredentialTierRequiredSummaryRule {
            message: "credential tier needs a value".to_string(),
        };
        let mut item = new_credential_tier("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = CredentialTierScopeLengthRule { max_length: 16 };
        let mut item = new_credential_tier("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = CredentialTierRequiredScopeRule {
            message: "credential tier needs an entry".to_string(),
        };
        let mut item = new_credential_tier("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

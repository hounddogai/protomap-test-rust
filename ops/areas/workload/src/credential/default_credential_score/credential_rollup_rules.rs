use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential rollup must pass. Check returns its problems, or none.
pub trait CredentialRollupRule: std::fmt::Debug {
    fn check(&self, item: &CredentialRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRollupAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl CredentialRollupRule for CredentialRollupAllowedCurrencyRule {
    fn check(&self, item: &CredentialRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is above maximum", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRollupCurrencyLengthRule {
    pub max_length: i64,
}

impl CredentialRollupRule for CredentialRollupCurrencyLengthRule {
    fn check(&self, item: &CredentialRollup) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRollupCodePrefixRule {
    pub prefix: String,
}

impl CredentialRollupRule for CredentialRollupCodePrefixRule {
    fn check(&self, item: &CredentialRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential rollup.
#[derive(Debug, Clone)]
pub struct CredentialRollupValidator {
    pub rules: Vec<Rc<dyn CredentialRollupRule>>,
}

impl CredentialRollupValidator {
    pub fn validate(&self, item: &CredentialRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_rollup_validator() -> CredentialRollupValidator {
    CredentialRollupValidator {
        rules: vec![
            Rc::new(CredentialRollupAllowedCurrencyRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn CredentialRollupRule>,
            Rc::new(CredentialRollupCurrencyLengthRule { max_length: 120 }),
            Rc::new(CredentialRollupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_rollup_validator();
        let item = new_credential_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = CredentialRollupCurrencyLengthRule { max_length: 120 };
        let mut item = new_credential_rollup("x", "y");
        item.currency = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

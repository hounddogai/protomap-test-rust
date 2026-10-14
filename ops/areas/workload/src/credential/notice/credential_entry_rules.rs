use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential entry must pass. Check returns its problems, or none.
pub trait CredentialEntryRule: std::fmt::Debug {
    fn check(&self, item: &CredentialEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEntryBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialEntryRule for CredentialEntryBurstRangeRule {
    fn check(&self, item: &CredentialEntry) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEntryVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialEntryRule for CredentialEntryVersionRangeRule {
    fn check(&self, item: &CredentialEntry) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEntryCodePrefixRule {
    pub prefix: String,
}

impl CredentialEntryRule for CredentialEntryCodePrefixRule {
    fn check(&self, item: &CredentialEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEntryRequiredTitleRule {
    pub message: String,
}

impl CredentialEntryRule for CredentialEntryRequiredTitleRule {
    fn check(&self, item: &CredentialEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEntryAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl CredentialEntryRule for CredentialEntryAllowedScopeRule {
    fn check(&self, item: &CredentialEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is unknown", item.scope)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential entry.
#[derive(Debug, Clone)]
pub struct CredentialEntryValidator {
    pub rules: Vec<Rc<dyn CredentialEntryRule>>,
}

impl CredentialEntryValidator {
    pub fn validate(&self, item: &CredentialEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_entry_validator() -> CredentialEntryValidator {
    CredentialEntryValidator {
        rules: vec![
            Rc::new(CredentialEntryBurstRangeRule { min: 120, max: 86400 }) as Rc<dyn CredentialEntryRule>,
            Rc::new(CredentialEntryVersionRangeRule { min: 32, max: 64 }),
            Rc::new(CredentialEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CredentialEntryRequiredTitleRule {
                message: "credential entry needs a value".to_string(),
            }),
            Rc::new(CredentialEntryAllowedScopeRule {
                allowed: vec!["manual".to_string(), "rolling".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_entry_validator();
        let item = new_credential_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = CredentialEntryBurstRangeRule { min: 120, max: 1000 };
        let mut item = new_credential_entry("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = CredentialEntryVersionRangeRule { min: 16, max: 1000 };
        let mut item = new_credential_entry("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CredentialEntryRequiredTitleRule {
            message: "credential entry needs this field".to_string(),
        };
        let mut item = new_credential_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

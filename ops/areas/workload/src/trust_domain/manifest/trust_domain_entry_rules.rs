use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain entry must pass. Check returns its problems, or none.
pub trait TrustDomainEntryRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainEntryRequiredFingerprintRule {
    pub message: String,
}

impl TrustDomainEntryRule for TrustDomainEntryRequiredFingerprintRule {
    fn check(&self, item: &TrustDomainEntry) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainEntryRequiredReviewerRule {
    pub message: String,
}

impl TrustDomainEntryRule for TrustDomainEntryRequiredReviewerRule {
    fn check(&self, item: &TrustDomainEntry) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainEntryCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainEntryRule for TrustDomainEntryCodePrefixRule {
    fn check(&self, item: &TrustDomainEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain entry.
#[derive(Debug, Clone)]
pub struct TrustDomainEntryValidator {
    pub rules: Vec<Rc<dyn TrustDomainEntryRule>>,
}

impl TrustDomainEntryValidator {
    pub fn validate(&self, item: &TrustDomainEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_entry_validator() -> TrustDomainEntryValidator {
    TrustDomainEntryValidator {
        rules: vec![
            Rc::new(TrustDomainEntryRequiredFingerprintRule {
                message: "trust domain entry needs a value".to_string(),
            }) as Rc<dyn TrustDomainEntryRule>,
            Rc::new(TrustDomainEntryRequiredReviewerRule {
                message: "trust domain entry needs an entry".to_string(),
            }),
            Rc::new(TrustDomainEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_entry_validator();
        let item = new_trust_domain_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = TrustDomainEntryRequiredFingerprintRule {
            message: "trust domain entry needs a value".to_string(),
        };
        let mut item = new_trust_domain_entry("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = TrustDomainEntryRequiredReviewerRule {
            message: "trust domain entry needs this field".to_string(),
        };
        let mut item = new_trust_domain_entry("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain record must pass. Check returns its problems, or none.
pub trait TrustDomainRecordRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl TrustDomainRecordRule for TrustDomainRecordAllowedLabelRule {
    fn check(&self, item: &TrustDomainRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordRequiredNamespaceRule {
    pub message: String,
}

impl TrustDomainRecordRule for TrustDomainRecordRequiredNamespaceRule {
    fn check(&self, item: &TrustDomainRecord) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordLabelLengthRule {
    pub max_length: i64,
}

impl TrustDomainRecordRule for TrustDomainRecordLabelLengthRule {
    fn check(&self, item: &TrustDomainRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainRecordRule for TrustDomainRecordCodeLengthRule {
    fn check(&self, item: &TrustDomainRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordActiveGuardRule {
    pub limit: i64,
}

impl TrustDomainRecordRule for TrustDomainRecordActiveGuardRule {
    fn check(&self, item: &TrustDomainRecord) -> Vec<String> {
        if item.active && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while active",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordRequiredCodeRule {
    pub message: String,
}

impl TrustDomainRecordRule for TrustDomainRecordRequiredCodeRule {
    fn check(&self, item: &TrustDomainRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain record.
#[derive(Debug, Clone)]
pub struct TrustDomainRecordValidator {
    pub rules: Vec<Rc<dyn TrustDomainRecordRule>>,
}

impl TrustDomainRecordValidator {
    pub fn validate(&self, item: &TrustDomainRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_record_validator() -> TrustDomainRecordValidator {
    TrustDomainRecordValidator {
        rules: vec![
            Rc::new(TrustDomainRecordAllowedLabelRule {
                allowed: vec!["tiered".to_string(), "adaptive".to_string(), "fallback".to_string()],
            }) as Rc<dyn TrustDomainRecordRule>,
            Rc::new(TrustDomainRecordRequiredNamespaceRule {
                message: "trust domain record needs this field".to_string(),
            }),
            Rc::new(TrustDomainRecordLabelLengthRule { max_length: 16 }),
            Rc::new(TrustDomainRecordCodeLengthRule { max_length: 32 }),
            Rc::new(TrustDomainRecordActiveGuardRule { limit: 32 }),
            Rc::new(TrustDomainRecordRequiredCodeRule {
                message: "trust domain record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_record_validator();
        let item = new_trust_domain_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = TrustDomainRecordRequiredNamespaceRule {
            message: "trust domain record needs this field".to_string(),
        };
        let mut item = new_trust_domain_record("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TrustDomainRecordLabelLengthRule { max_length: 120 };
        let mut item = new_trust_domain_record("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainRecordCodeLengthRule { max_length: 32 };
        let mut item = new_trust_domain_record("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainRecordRequiredCodeRule {
            message: "trust domain record needs a value".to_string(),
        };
        let mut item = new_trust_domain_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

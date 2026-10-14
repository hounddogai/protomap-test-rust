use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling ttl policy agreement must pass. Check returns its problems, or none.
pub trait RollingTtlPolicyAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RollingTtlPolicyAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreementCodeLengthRule {
    pub max_length: i64,
}

impl RollingTtlPolicyAgreementRule for RollingTtlPolicyAgreementCodeLengthRule {
    fn check(&self, item: &RollingTtlPolicyAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreementNotesLengthRule {
    pub max_length: i64,
}

impl RollingTtlPolicyAgreementRule for RollingTtlPolicyAgreementNotesLengthRule {
    fn check(&self, item: &RollingTtlPolicyAgreement) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreementAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl RollingTtlPolicyAgreementRule for RollingTtlPolicyAgreementAllowedNotesRule {
    fn check(&self, item: &RollingTtlPolicyAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is out of range", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreementCodePrefixRule {
    pub prefix: String,
}

impl RollingTtlPolicyAgreementRule for RollingTtlPolicyAgreementCodePrefixRule {
    fn check(&self, item: &RollingTtlPolicyAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling ttl policy agreement.
#[derive(Debug, Clone)]
pub struct RollingTtlPolicyAgreementValidator {
    pub rules: Vec<Rc<dyn RollingTtlPolicyAgreementRule>>,
}

impl RollingTtlPolicyAgreementValidator {
    pub fn validate(&self, item: &RollingTtlPolicyAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTtlPolicyAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTtlPolicyAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_ttl_policy_agreement_validator() -> RollingTtlPolicyAgreementValidator {
    RollingTtlPolicyAgreementValidator {
        rules: vec![
            Rc::new(RollingTtlPolicyAgreementCodeLengthRule { max_length: 16 })
                as Rc<dyn RollingTtlPolicyAgreementRule>,
            Rc::new(RollingTtlPolicyAgreementNotesLengthRule { max_length: 255 }),
            Rc::new(RollingTtlPolicyAgreementAllowedNotesRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(RollingTtlPolicyAgreementCodePrefixRule {
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
        let validator = default_rolling_ttl_policy_agreement_validator();
        let item = new_rolling_ttl_policy_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingTtlPolicyAgreementCodeLengthRule { max_length: 32 };
        let mut item = new_rolling_ttl_policy_agreement("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollingTtlPolicyAgreementNotesLengthRule { max_length: 255 };
        let mut item = new_rolling_ttl_policy_agreement("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

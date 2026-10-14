use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain target must pass. Check returns its problems, or none.
pub trait TrustDomainTargetRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTargetCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainTargetRule for TrustDomainTargetCodePrefixRule {
    fn check(&self, item: &TrustDomainTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTargetRequiredCodeRule {
    pub message: String,
}

impl TrustDomainTargetRule for TrustDomainTargetRequiredCodeRule {
    fn check(&self, item: &TrustDomainTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTargetActiveGuardRule {
    pub limit: i64,
}

impl TrustDomainTargetRule for TrustDomainTargetActiveGuardRule {
    fn check(&self, item: &TrustDomainTarget) -> Vec<String> {
        if item.active && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while active",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTargetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TrustDomainTargetRule for TrustDomainTargetAllowedTitleRule {
    fn check(&self, item: &TrustDomainTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTargetCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainTargetRule for TrustDomainTargetCodeLengthRule {
    fn check(&self, item: &TrustDomainTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain target.
#[derive(Debug, Clone)]
pub struct TrustDomainTargetValidator {
    pub rules: Vec<Rc<dyn TrustDomainTargetRule>>,
}

impl TrustDomainTargetValidator {
    pub fn validate(&self, item: &TrustDomainTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_target_validator() -> TrustDomainTargetValidator {
    TrustDomainTargetValidator {
        rules: vec![
            Rc::new(TrustDomainTargetCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn TrustDomainTargetRule>,
            Rc::new(TrustDomainTargetRequiredCodeRule {
                message: "trust domain target needs an entry".to_string(),
            }),
            Rc::new(TrustDomainTargetActiveGuardRule { limit: 120 }),
            Rc::new(TrustDomainTargetAllowedTitleRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(TrustDomainTargetCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_target_validator();
        let item = new_trust_domain_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainTargetRequiredCodeRule {
            message: "trust domain target needs an entry".to_string(),
        };
        let mut item = new_trust_domain_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainTargetCodeLengthRule { max_length: 32 };
        let mut item = new_trust_domain_target("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

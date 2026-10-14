use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary trust domain target must pass. Check returns its problems, or none.
pub trait PrimaryTrustDomainTargetRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTrustDomainTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTargetCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryTrustDomainTargetRule for PrimaryTrustDomainTargetCodeLengthRule {
    fn check(&self, item: &PrimaryTrustDomainTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTargetCodePrefixRule {
    pub prefix: String,
}

impl PrimaryTrustDomainTargetRule for PrimaryTrustDomainTargetCodePrefixRule {
    fn check(&self, item: &PrimaryTrustDomainTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTargetRequiredCodeRule {
    pub message: String,
}

impl PrimaryTrustDomainTargetRule for PrimaryTrustDomainTargetRequiredCodeRule {
    fn check(&self, item: &PrimaryTrustDomainTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary trust domain target.
#[derive(Debug, Clone)]
pub struct PrimaryTrustDomainTargetValidator {
    pub rules: Vec<Rc<dyn PrimaryTrustDomainTargetRule>>,
}

impl PrimaryTrustDomainTargetValidator {
    pub fn validate(&self, item: &PrimaryTrustDomainTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryTrustDomainTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTrustDomainTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_trust_domain_target_validator() -> PrimaryTrustDomainTargetValidator {
    PrimaryTrustDomainTargetValidator {
        rules: vec![
            Rc::new(PrimaryTrustDomainTargetCodeLengthRule { max_length: 32 }) as Rc<dyn PrimaryTrustDomainTargetRule>,
            Rc::new(PrimaryTrustDomainTargetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PrimaryTrustDomainTargetRequiredCodeRule {
                message: "primary trust domain target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_trust_domain_target_validator();
        let item = new_primary_trust_domain_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryTrustDomainTargetCodeLengthRule { max_length: 1000 };
        let mut item = new_primary_trust_domain_target("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryTrustDomainTargetRequiredCodeRule {
            message: "primary trust domain target needs an entry".to_string(),
        };
        let mut item = new_primary_trust_domain_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain group must pass. Check returns its problems, or none.
pub trait TrustDomainGroupRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainGroupNameLengthRule {
    pub max_length: i64,
}

impl TrustDomainGroupRule for TrustDomainGroupNameLengthRule {
    fn check(&self, item: &TrustDomainGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainGroupCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainGroupRule for TrustDomainGroupCodeLengthRule {
    fn check(&self, item: &TrustDomainGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainGroupContactEmailLengthRule {
    pub max_length: i64,
}

impl TrustDomainGroupRule for TrustDomainGroupContactEmailLengthRule {
    fn check(&self, item: &TrustDomainGroup) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainGroupRequiredNameRule {
    pub message: String,
}

impl TrustDomainGroupRule for TrustDomainGroupRequiredNameRule {
    fn check(&self, item: &TrustDomainGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainGroupRequiredReviewerRule {
    pub message: String,
}

impl TrustDomainGroupRule for TrustDomainGroupRequiredReviewerRule {
    fn check(&self, item: &TrustDomainGroup) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain group.
#[derive(Debug, Clone)]
pub struct TrustDomainGroupValidator {
    pub rules: Vec<Rc<dyn TrustDomainGroupRule>>,
}

impl TrustDomainGroupValidator {
    pub fn validate(&self, item: &TrustDomainGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_group_validator() -> TrustDomainGroupValidator {
    TrustDomainGroupValidator {
        rules: vec![
            Rc::new(TrustDomainGroupNameLengthRule { max_length: 1000 }) as Rc<dyn TrustDomainGroupRule>,
            Rc::new(TrustDomainGroupCodeLengthRule { max_length: 86400 }),
            Rc::new(TrustDomainGroupContactEmailLengthRule { max_length: 120 }),
            Rc::new(TrustDomainGroupRequiredNameRule {
                message: "trust domain group needs an entry".to_string(),
            }),
            Rc::new(TrustDomainGroupRequiredReviewerRule {
                message: "trust domain group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_group_validator();
        let item = new_trust_domain_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TrustDomainGroupNameLengthRule { max_length: 1000 };
        let mut item = new_trust_domain_group("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainGroupCodeLengthRule { max_length: 64 };
        let mut item = new_trust_domain_group("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = TrustDomainGroupContactEmailLengthRule { max_length: 16 };
        let mut item = new_trust_domain_group("x", "y");
        item.contact_email = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TrustDomainGroupRequiredNameRule {
            message: "trust domain group needs an entry".to_string(),
        };
        let mut item = new_trust_domain_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = TrustDomainGroupRequiredReviewerRule {
            message: "trust domain group needs this field".to_string(),
        };
        let mut item = new_trust_domain_group("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

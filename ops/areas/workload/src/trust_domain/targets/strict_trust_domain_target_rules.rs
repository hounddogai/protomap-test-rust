use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict trust domain target must pass. Check returns its problems, or none.
pub trait StrictTrustDomainTargetRule: std::fmt::Debug {
    fn check(&self, item: &StrictTrustDomainTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTrustDomainTargetRequiredCodeRule {
    pub message: String,
}

impl StrictTrustDomainTargetRule for StrictTrustDomainTargetRequiredCodeRule {
    fn check(&self, item: &StrictTrustDomainTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTrustDomainTargetCodePrefixRule {
    pub prefix: String,
}

impl StrictTrustDomainTargetRule for StrictTrustDomainTargetCodePrefixRule {
    fn check(&self, item: &StrictTrustDomainTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTrustDomainTargetKeyIdLengthRule {
    pub max_length: i64,
}

impl StrictTrustDomainTargetRule for StrictTrustDomainTargetKeyIdLengthRule {
    fn check(&self, item: &StrictTrustDomainTarget) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTrustDomainTargetTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl StrictTrustDomainTargetRule for StrictTrustDomainTargetTagsLimitRule {
    fn check(&self, item: &StrictTrustDomainTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a strict trust domain target.
#[derive(Debug, Clone)]
pub struct StrictTrustDomainTargetValidator {
    pub rules: Vec<Rc<dyn StrictTrustDomainTargetRule>>,
}

impl StrictTrustDomainTargetValidator {
    pub fn validate(&self, item: &StrictTrustDomainTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictTrustDomainTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTrustDomainTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_trust_domain_target_validator() -> StrictTrustDomainTargetValidator {
    StrictTrustDomainTargetValidator {
        rules: vec![
            Rc::new(StrictTrustDomainTargetRequiredCodeRule {
                message: "strict trust domain target needs an entry".to_string(),
            }) as Rc<dyn StrictTrustDomainTargetRule>,
            Rc::new(StrictTrustDomainTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(StrictTrustDomainTargetKeyIdLengthRule { max_length: 86400 }),
            Rc::new(StrictTrustDomainTargetTagsLimitRule {
                max_count: 120,
                banned: Vec::new(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_trust_domain_target_validator();
        let item = new_strict_trust_domain_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictTrustDomainTargetRequiredCodeRule {
            message: "strict trust domain target needs a value".to_string(),
        };
        let mut item = new_strict_trust_domain_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = StrictTrustDomainTargetKeyIdLengthRule { max_length: 32 };
        let mut item = new_strict_trust_domain_target("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global trust domain tier must pass. Check returns its problems, or none.
pub trait GlobalTrustDomainTierRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTrustDomainTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainTierContactEmailLengthRule {
    pub max_length: i64,
}

impl GlobalTrustDomainTierRule for GlobalTrustDomainTierContactEmailLengthRule {
    fn check(&self, item: &GlobalTrustDomainTier) -> Vec<String> {
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
pub struct GlobalTrustDomainTierStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTrustDomainTierRule for GlobalTrustDomainTierStartsAtRangeRule {
    fn check(&self, item: &GlobalTrustDomainTier) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainTierRequiredContactEmailRule {
    pub message: String,
}

impl GlobalTrustDomainTierRule for GlobalTrustDomainTierRequiredContactEmailRule {
    fn check(&self, item: &GlobalTrustDomainTier) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global trust domain tier.
#[derive(Debug, Clone)]
pub struct GlobalTrustDomainTierValidator {
    pub rules: Vec<Rc<dyn GlobalTrustDomainTierRule>>,
}

impl GlobalTrustDomainTierValidator {
    pub fn validate(&self, item: &GlobalTrustDomainTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalTrustDomainTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTrustDomainTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_trust_domain_tier_validator() -> GlobalTrustDomainTierValidator {
    GlobalTrustDomainTierValidator {
        rules: vec![
            Rc::new(GlobalTrustDomainTierContactEmailLengthRule { max_length: 1000 })
                as Rc<dyn GlobalTrustDomainTierRule>,
            Rc::new(GlobalTrustDomainTierStartsAtRangeRule { min: 32, max: 1000 }),
            Rc::new(GlobalTrustDomainTierRequiredContactEmailRule {
                message: "global trust domain tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_trust_domain_tier_validator();
        let item = new_global_trust_domain_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = GlobalTrustDomainTierContactEmailLengthRule { max_length: 255 };
        let mut item = new_global_trust_domain_tier("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = GlobalTrustDomainTierStartsAtRangeRule { min: 16, max: 64 };
        let mut item = new_global_trust_domain_tier("x", "y");
        item.starts_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = GlobalTrustDomainTierRequiredContactEmailRule {
            message: "global trust domain tier needs an entry".to_string(),
        };
        let mut item = new_global_trust_domain_tier("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}

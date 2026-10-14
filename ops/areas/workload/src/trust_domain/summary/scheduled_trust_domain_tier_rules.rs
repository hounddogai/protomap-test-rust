use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled trust domain tier must pass. Check returns its problems, or none.
pub trait ScheduledTrustDomainTierRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTrustDomainTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledTrustDomainTierRule for ScheduledTrustDomainTierAllowedCodeRule {
    fn check(&self, item: &ScheduledTrustDomainTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainTierNameLengthRule {
    pub max_length: i64,
}

impl ScheduledTrustDomainTierRule for ScheduledTrustDomainTierNameLengthRule {
    fn check(&self, item: &ScheduledTrustDomainTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainTierSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledTrustDomainTierRule for ScheduledTrustDomainTierSequenceRangeRule {
    fn check(&self, item: &ScheduledTrustDomainTier) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainTierMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledTrustDomainTierRule for ScheduledTrustDomainTierMaximumRangeRule {
    fn check(&self, item: &ScheduledTrustDomainTier) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled trust domain tier.
#[derive(Debug, Clone)]
pub struct ScheduledTrustDomainTierValidator {
    pub rules: Vec<Rc<dyn ScheduledTrustDomainTierRule>>,
}

impl ScheduledTrustDomainTierValidator {
    pub fn validate(&self, item: &ScheduledTrustDomainTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledTrustDomainTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTrustDomainTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_trust_domain_tier_validator() -> ScheduledTrustDomainTierValidator {
    ScheduledTrustDomainTierValidator {
        rules: vec![
            Rc::new(ScheduledTrustDomainTierAllowedCodeRule {
                allowed: vec!["global".to_string()],
            }) as Rc<dyn ScheduledTrustDomainTierRule>,
            Rc::new(ScheduledTrustDomainTierNameLengthRule { max_length: 64 }),
            Rc::new(ScheduledTrustDomainTierSequenceRangeRule { min: 120, max: 1000 }),
            Rc::new(ScheduledTrustDomainTierMaximumRangeRule { min: 6.1, max: 62.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_trust_domain_tier_validator();
        let item = new_scheduled_trust_domain_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledTrustDomainTierNameLengthRule { max_length: 32 };
        let mut item = new_scheduled_trust_domain_tier("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ScheduledTrustDomainTierSequenceRangeRule { min: 64, max: 80 };
        let mut item = new_scheduled_trust_domain_tier("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ScheduledTrustDomainTierMaximumRangeRule { min: 1.7, max: 3112.9 };
        let mut item = new_scheduled_trust_domain_tier("x", "y");
        item.maximum = 3114.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}

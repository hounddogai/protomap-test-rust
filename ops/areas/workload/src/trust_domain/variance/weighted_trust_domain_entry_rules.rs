use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted trust domain entry must pass. Check returns its problems, or none.
pub trait WeightedTrustDomainEntryRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTrustDomainEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainEntryCodePrefixRule {
    pub prefix: String,
}

impl WeightedTrustDomainEntryRule for WeightedTrustDomainEntryCodePrefixRule {
    fn check(&self, item: &WeightedTrustDomainEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainEntryAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl WeightedTrustDomainEntryRule for WeightedTrustDomainEntryAllowedLabelRule {
    fn check(&self, item: &WeightedTrustDomainEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainEntryRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedTrustDomainEntryRule for WeightedTrustDomainEntryRatioRangeRule {
    fn check(&self, item: &WeightedTrustDomainEntry) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainEntryTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedTrustDomainEntryRule for WeightedTrustDomainEntryTtlSecondsRangeRule {
    fn check(&self, item: &WeightedTrustDomainEntry) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted trust domain entry.
#[derive(Debug, Clone)]
pub struct WeightedTrustDomainEntryValidator {
    pub rules: Vec<Rc<dyn WeightedTrustDomainEntryRule>>,
}

impl WeightedTrustDomainEntryValidator {
    pub fn validate(&self, item: &WeightedTrustDomainEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedTrustDomainEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTrustDomainEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_trust_domain_entry_validator() -> WeightedTrustDomainEntryValidator {
    WeightedTrustDomainEntryValidator {
        rules: vec![
            Rc::new(WeightedTrustDomainEntryCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn WeightedTrustDomainEntryRule>,
            Rc::new(WeightedTrustDomainEntryAllowedLabelRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(WeightedTrustDomainEntryRatioRangeRule { min: 7.2, max: 1703.2 }),
            Rc::new(WeightedTrustDomainEntryTtlSecondsRangeRule { min: 255, max: 1000 }),
        ],
    }
}

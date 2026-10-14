use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict federation profile must pass. Check returns its problems, or none.
pub trait StrictFederationProfileRule: std::fmt::Debug {
    fn check(&self, item: &StrictFederationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationProfileHostLengthRule {
    pub max_length: i64,
}

impl StrictFederationProfileRule for StrictFederationProfileHostLengthRule {
    fn check(&self, item: &StrictFederationProfile) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationProfileAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl StrictFederationProfileRule for StrictFederationProfileAllowedCategoryRule {
    fn check(&self, item: &StrictFederationProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is missing", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationProfileAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictFederationProfileRule for StrictFederationProfileAmountRangeRule {
    fn check(&self, item: &StrictFederationProfile) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict federation profile.
#[derive(Debug, Clone)]
pub struct StrictFederationProfileValidator {
    pub rules: Vec<Rc<dyn StrictFederationProfileRule>>,
}

impl StrictFederationProfileValidator {
    pub fn validate(&self, item: &StrictFederationProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictFederationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFederationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_federation_profile_validator() -> StrictFederationProfileValidator {
    StrictFederationProfileValidator {
        rules: vec![
            Rc::new(StrictFederationProfileHostLengthRule { max_length: 16 }) as Rc<dyn StrictFederationProfileRule>,
            Rc::new(StrictFederationProfileAllowedCategoryRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(StrictFederationProfileAmountRangeRule { min: 5.0, max: 4002.4 }),
        ],
    }
}

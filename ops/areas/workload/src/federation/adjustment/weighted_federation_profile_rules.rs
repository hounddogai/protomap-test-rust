use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted federation profile must pass. Check returns its problems, or none.
pub trait WeightedFederationProfileRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFederationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationProfileRequiredCodeRule {
    pub message: String,
}

impl WeightedFederationProfileRule for WeightedFederationProfileRequiredCodeRule {
    fn check(&self, item: &WeightedFederationProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationProfileFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedFederationProfileRule for WeightedFederationProfileFloorValueRangeRule {
    fn check(&self, item: &WeightedFederationProfile) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationProfileCodeLengthRule {
    pub max_length: i64,
}

impl WeightedFederationProfileRule for WeightedFederationProfileCodeLengthRule {
    fn check(&self, item: &WeightedFederationProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationProfileNameLengthRule {
    pub max_length: i64,
}

impl WeightedFederationProfileRule for WeightedFederationProfileNameLengthRule {
    fn check(&self, item: &WeightedFederationProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted federation profile.
#[derive(Debug, Clone)]
pub struct WeightedFederationProfileValidator {
    pub rules: Vec<Rc<dyn WeightedFederationProfileRule>>,
}

impl WeightedFederationProfileValidator {
    pub fn validate(&self, item: &WeightedFederationProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedFederationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFederationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_federation_profile_validator() -> WeightedFederationProfileValidator {
    WeightedFederationProfileValidator {
        rules: vec![
            Rc::new(WeightedFederationProfileRequiredCodeRule {
                message: "weighted federation profile needs a value".to_string(),
            }) as Rc<dyn WeightedFederationProfileRule>,
            Rc::new(WeightedFederationProfileFloorValueRangeRule { min: 9.6, max: 3484.4 }),
            Rc::new(WeightedFederationProfileCodeLengthRule { max_length: 80 }),
            Rc::new(WeightedFederationProfileNameLengthRule { max_length: 3600 }),
        ],
    }
}

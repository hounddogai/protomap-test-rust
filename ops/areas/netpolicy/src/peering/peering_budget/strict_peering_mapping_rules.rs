use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict peering mapping must pass. Check returns its problems, or none.
pub trait StrictPeeringMappingRule: std::fmt::Debug {
    fn check(&self, item: &StrictPeeringMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringMappingRequiredCountryRule {
    pub message: String,
}

impl StrictPeeringMappingRule for StrictPeeringMappingRequiredCountryRule {
    fn check(&self, item: &StrictPeeringMapping) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringMappingRequiredCodeRule {
    pub message: String,
}

impl StrictPeeringMappingRule for StrictPeeringMappingRequiredCodeRule {
    fn check(&self, item: &StrictPeeringMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringMappingChecksumLengthRule {
    pub max_length: i64,
}

impl StrictPeeringMappingRule for StrictPeeringMappingChecksumLengthRule {
    fn check(&self, item: &StrictPeeringMapping) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringMappingPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPeeringMappingRule for StrictPeeringMappingPortRangeRule {
    fn check(&self, item: &StrictPeeringMapping) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict peering mapping.
#[derive(Debug, Clone)]
pub struct StrictPeeringMappingValidator {
    pub rules: Vec<Rc<dyn StrictPeeringMappingRule>>,
}

impl StrictPeeringMappingValidator {
    pub fn validate(&self, item: &StrictPeeringMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictPeeringMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPeeringMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_peering_mapping_validator() -> StrictPeeringMappingValidator {
    StrictPeeringMappingValidator {
        rules: vec![
            Rc::new(StrictPeeringMappingRequiredCountryRule {
                message: "strict peering mapping needs this field".to_string(),
            }) as Rc<dyn StrictPeeringMappingRule>,
            Rc::new(StrictPeeringMappingRequiredCodeRule {
                message: "strict peering mapping needs an entry".to_string(),
            }),
            Rc::new(StrictPeeringMappingChecksumLengthRule { max_length: 32 }),
            Rc::new(StrictPeeringMappingPortRangeRule { min: 16, max: 1000 }),
        ],
    }
}

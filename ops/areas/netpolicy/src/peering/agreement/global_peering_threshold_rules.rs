use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global peering threshold must pass. Check returns its problems, or none.
pub trait GlobalPeeringThresholdRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPeeringThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringThresholdPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPeeringThresholdRule for GlobalPeeringThresholdPortRangeRule {
    fn check(&self, item: &GlobalPeeringThreshold) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringThresholdRequiredNameRule {
    pub message: String,
}

impl GlobalPeeringThresholdRule for GlobalPeeringThresholdRequiredNameRule {
    fn check(&self, item: &GlobalPeeringThreshold) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringThresholdPhoneLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringThresholdRule for GlobalPeeringThresholdPhoneLengthRule {
    fn check(&self, item: &GlobalPeeringThreshold) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global peering threshold.
#[derive(Debug, Clone)]
pub struct GlobalPeeringThresholdValidator {
    pub rules: Vec<Rc<dyn GlobalPeeringThresholdRule>>,
}

impl GlobalPeeringThresholdValidator {
    pub fn validate(&self, item: &GlobalPeeringThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalPeeringThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPeeringThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_peering_threshold_validator() -> GlobalPeeringThresholdValidator {
    GlobalPeeringThresholdValidator {
        rules: vec![
            Rc::new(GlobalPeeringThresholdPortRangeRule { min: 32, max: 86400 }) as Rc<dyn GlobalPeeringThresholdRule>,
            Rc::new(GlobalPeeringThresholdRequiredNameRule {
                message: "global peering threshold needs this field".to_string(),
            }),
            Rc::new(GlobalPeeringThresholdPhoneLengthRule { max_length: 80 }),
        ],
    }
}

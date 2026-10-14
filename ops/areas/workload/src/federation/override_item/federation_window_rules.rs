use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation window must pass. Check returns its problems, or none.
pub trait FederationWindowRule: std::fmt::Debug {
    fn check(&self, item: &FederationWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationWindowRequiredCodeRule {
    pub message: String,
}

impl FederationWindowRule for FederationWindowRequiredCodeRule {
    fn check(&self, item: &FederationWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationWindowLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationWindowRule for FederationWindowLimitRangeRule {
    fn check(&self, item: &FederationWindow) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationWindowCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationWindowRule for FederationWindowCreatedAtRangeRule {
    fn check(&self, item: &FederationWindow) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationWindowCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationWindowRule for FederationWindowCostRangeRule {
    fn check(&self, item: &FederationWindow) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation window.
#[derive(Debug, Clone)]
pub struct FederationWindowValidator {
    pub rules: Vec<Rc<dyn FederationWindowRule>>,
}

impl FederationWindowValidator {
    pub fn validate(&self, item: &FederationWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_window_validator() -> FederationWindowValidator {
    FederationWindowValidator {
        rules: vec![
            Rc::new(FederationWindowRequiredCodeRule {
                message: "federation window needs an entry".to_string(),
            }) as Rc<dyn FederationWindowRule>,
            Rc::new(FederationWindowLimitRangeRule { min: 32, max: 80 }),
            Rc::new(FederationWindowCreatedAtRangeRule { min: 16, max: 64 }),
            Rc::new(FederationWindowCostRangeRule { min: 9.1, max: 4801.1 }),
        ],
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query adjustment must pass. Check returns its problems, or none.
pub trait QueryAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &QueryAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAdjustmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryAdjustmentRule for QueryAdjustmentRateRangeRule {
    fn check(&self, item: &QueryAdjustment) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl QueryAdjustmentRule for QueryAdjustmentCodeLengthRule {
    fn check(&self, item: &QueryAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAdjustmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryAdjustmentRule for QueryAdjustmentDiscountRangeRule {
    fn check(&self, item: &QueryAdjustment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAdjustmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryAdjustmentRule for QueryAdjustmentFloorValueRangeRule {
    fn check(&self, item: &QueryAdjustment) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAdjustmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryAdjustmentRule for QueryAdjustmentLimitRangeRule {
    fn check(&self, item: &QueryAdjustment) -> Vec<String> {
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

/// Runs every rule of a query adjustment.
#[derive(Debug, Clone)]
pub struct QueryAdjustmentValidator {
    pub rules: Vec<Rc<dyn QueryAdjustmentRule>>,
}

impl QueryAdjustmentValidator {
    pub fn validate(&self, item: &QueryAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_adjustment_validator() -> QueryAdjustmentValidator {
    QueryAdjustmentValidator {
        rules: vec![
            Rc::new(QueryAdjustmentRateRangeRule { min: 2.6, max: 4838.4 }) as Rc<dyn QueryAdjustmentRule>,
            Rc::new(QueryAdjustmentCodeLengthRule { max_length: 86400 }),
            Rc::new(QueryAdjustmentDiscountRangeRule { min: 8.7, max: 113.8 }),
            Rc::new(QueryAdjustmentFloorValueRangeRule { min: 5.2, max: 1963.4 }),
            Rc::new(QueryAdjustmentLimitRangeRule { min: 32, max: 64 }),
        ],
    }
}

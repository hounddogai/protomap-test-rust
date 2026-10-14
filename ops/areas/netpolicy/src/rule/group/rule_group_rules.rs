use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule group must pass. Check returns its problems, or none.
pub trait RuleGroupRule: std::fmt::Debug {
    fn check(&self, item: &RuleGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleGroupDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleGroupRule for RuleGroupDiscountRangeRule {
    fn check(&self, item: &RuleGroup) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleGroupFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleGroupRule for RuleGroupFloorValueRangeRule {
    fn check(&self, item: &RuleGroup) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleGroupRequiredChannelRule {
    pub message: String,
}

impl RuleGroupRule for RuleGroupRequiredChannelRule {
    fn check(&self, item: &RuleGroup) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule group.
#[derive(Debug, Clone)]
pub struct RuleGroupValidator {
    pub rules: Vec<Rc<dyn RuleGroupRule>>,
}

impl RuleGroupValidator {
    pub fn validate(&self, item: &RuleGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_group_validator() -> RuleGroupValidator {
    RuleGroupValidator {
        rules: vec![
            Rc::new(RuleGroupDiscountRangeRule { min: 8.6, max: 2360.4 }) as Rc<dyn RuleGroupRule>,
            Rc::new(RuleGroupFloorValueRangeRule { min: 5.5, max: 1658.1 }),
            Rc::new(RuleGroupRequiredChannelRule {
                message: "rule group needs a value".to_string(),
            }),
        ],
    }
}

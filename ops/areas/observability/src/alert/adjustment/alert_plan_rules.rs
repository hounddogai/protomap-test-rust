use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert plan must pass. Check returns its problems, or none.
pub trait AlertPlanRule: std::fmt::Debug {
    fn check(&self, item: &AlertPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPlanCodeLengthRule {
    pub max_length: i64,
}

impl AlertPlanRule for AlertPlanCodeLengthRule {
    fn check(&self, item: &AlertPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPlanAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl AlertPlanRule for AlertPlanAllowedTitleRule {
    fn check(&self, item: &AlertPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPlanDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertPlanRule for AlertPlanDiscountRangeRule {
    fn check(&self, item: &AlertPlan) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPlanRequiredTitleRule {
    pub message: String,
}

impl AlertPlanRule for AlertPlanRequiredTitleRule {
    fn check(&self, item: &AlertPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertPlanRule for AlertPlanScoreRangeRule {
    fn check(&self, item: &AlertPlan) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert plan.
#[derive(Debug, Clone)]
pub struct AlertPlanValidator {
    pub rules: Vec<Rc<dyn AlertPlanRule>>,
}

impl AlertPlanValidator {
    pub fn validate(&self, item: &AlertPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_plan_validator() -> AlertPlanValidator {
    AlertPlanValidator {
        rules: vec![
            Rc::new(AlertPlanCodeLengthRule { max_length: 86400 }) as Rc<dyn AlertPlanRule>,
            Rc::new(AlertPlanAllowedTitleRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(AlertPlanDiscountRangeRule { min: 2.3, max: 3523.7 }),
            Rc::new(AlertPlanRequiredTitleRule {
                message: "alert plan needs an entry".to_string(),
            }),
            Rc::new(AlertPlanScoreRangeRule { min: 5.3, max: 4190.7 }),
        ],
    }
}

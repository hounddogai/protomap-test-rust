use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant plan must pass. Check returns its problems, or none.
pub trait GrantPlanRule: std::fmt::Debug {
    fn check(&self, item: &GrantPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanRequiredTitleRule {
    pub message: String,
}

impl GrantPlanRule for GrantPlanRequiredTitleRule {
    fn check(&self, item: &GrantPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanTitleLengthRule {
    pub max_length: i64,
}

impl GrantPlanRule for GrantPlanTitleLengthRule {
    fn check(&self, item: &GrantPlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantPlanRule for GrantPlanBurstRangeRule {
    fn check(&self, item: &GrantPlan) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantPlanRule for GrantPlanCeilingRangeRule {
    fn check(&self, item: &GrantPlan) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanRequiredCodeRule {
    pub message: String,
}

impl GrantPlanRule for GrantPlanRequiredCodeRule {
    fn check(&self, item: &GrantPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanScopeLengthRule {
    pub max_length: i64,
}

impl GrantPlanRule for GrantPlanScopeLengthRule {
    fn check(&self, item: &GrantPlan) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant plan.
#[derive(Debug, Clone)]
pub struct GrantPlanValidator {
    pub rules: Vec<Rc<dyn GrantPlanRule>>,
}

impl GrantPlanValidator {
    pub fn validate(&self, item: &GrantPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_plan_validator() -> GrantPlanValidator {
    GrantPlanValidator {
        rules: vec![
            Rc::new(GrantPlanRequiredTitleRule {
                message: "grant plan needs a value".to_string(),
            }) as Rc<dyn GrantPlanRule>,
            Rc::new(GrantPlanTitleLengthRule { max_length: 1000 }),
            Rc::new(GrantPlanBurstRangeRule { min: 80, max: 86400 }),
            Rc::new(GrantPlanCeilingRangeRule { min: 2.8, max: 650.0 }),
            Rc::new(GrantPlanRequiredCodeRule {
                message: "grant plan needs a value".to_string(),
            }),
            Rc::new(GrantPlanScopeLengthRule { max_length: 1000 }),
        ],
    }
}

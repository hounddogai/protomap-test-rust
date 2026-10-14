use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling plan checkpoint must pass. Check returns its problems, or none.
pub trait RollingPlanCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RollingPlanCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanCheckpointClusterLengthRule {
    pub max_length: i64,
}

impl RollingPlanCheckpointRule for RollingPlanCheckpointClusterLengthRule {
    fn check(&self, item: &RollingPlanCheckpoint) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanCheckpointVisibleGuardRule {
    pub limit: f64,
}

impl RollingPlanCheckpointRule for RollingPlanCheckpointVisibleGuardRule {
    fn check(&self, item: &RollingPlanCheckpoint) -> Vec<String> {
        if item.visible && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while visible", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanCheckpointAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl RollingPlanCheckpointRule for RollingPlanCheckpointAllowedTenantIdRule {
    fn check(&self, item: &RollingPlanCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is below minimum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanCheckpointBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingPlanCheckpointRule for RollingPlanCheckpointBudgetRangeRule {
    fn check(&self, item: &RollingPlanCheckpoint) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling plan checkpoint.
#[derive(Debug, Clone)]
pub struct RollingPlanCheckpointValidator {
    pub rules: Vec<Rc<dyn RollingPlanCheckpointRule>>,
}

impl RollingPlanCheckpointValidator {
    pub fn validate(&self, item: &RollingPlanCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPlanCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPlanCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_plan_checkpoint_validator() -> RollingPlanCheckpointValidator {
    RollingPlanCheckpointValidator {
        rules: vec![
            Rc::new(RollingPlanCheckpointClusterLengthRule { max_length: 86400 }) as Rc<dyn RollingPlanCheckpointRule>,
            Rc::new(RollingPlanCheckpointVisibleGuardRule { limit: 4.8 }),
            Rc::new(RollingPlanCheckpointAllowedTenantIdRule {
                allowed: vec!["capped".to_string(), "manual".to_string()],
            }),
            Rc::new(RollingPlanCheckpointBudgetRangeRule { min: 4.0, max: 3903.0 }),
        ],
    }
}

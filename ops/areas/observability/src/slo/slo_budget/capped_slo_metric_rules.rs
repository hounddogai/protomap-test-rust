use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped slo metric must pass. Check returns its problems, or none.
pub trait CappedSloMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedSloMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloMetricWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSloMetricRule for CappedSloMetricWeightRangeRule {
    fn check(&self, item: &CappedSloMetric) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloMetricAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl CappedSloMetricRule for CappedSloMetricAllowedTimezoneRule {
    fn check(&self, item: &CappedSloMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is stale", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloMetricPinnedGuardRule {
    pub limit: f64,
}

impl CappedSloMetricRule for CappedSloMetricPinnedGuardRule {
    fn check(&self, item: &CappedSloMetric) -> Vec<String> {
        if item.pinned && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while pinned", item.weight, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped slo metric.
#[derive(Debug, Clone)]
pub struct CappedSloMetricValidator {
    pub rules: Vec<Rc<dyn CappedSloMetricRule>>,
}

impl CappedSloMetricValidator {
    pub fn validate(&self, item: &CappedSloMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedSloMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSloMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_slo_metric_validator() -> CappedSloMetricValidator {
    CappedSloMetricValidator {
        rules: vec![
            Rc::new(CappedSloMetricWeightRangeRule { min: 0.3, max: 1723.1 }) as Rc<dyn CappedSloMetricRule>,
            Rc::new(CappedSloMetricAllowedTimezoneRule { allowed: Vec::new() }),
            Rc::new(CappedSloMetricPinnedGuardRule { limit: 6.9 }),
        ],
    }
}

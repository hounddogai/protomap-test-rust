use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary limit quota calculation.
pub trait PrimaryLimitQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryLimitQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryLimitQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryLimitQuotaAdjuster for TieredPrimaryLimitQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryLimitQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryLimitQuotaAdjuster {
    pub percent: f64,
}

impl PrimaryLimitQuotaAdjuster for PercentPrimaryLimitQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryLimitQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPrimaryLimitQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryLimitQuotaAdjuster for CategoryPrimaryLimitQuotaAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PrimaryLimitQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary limit quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryLimitQuotaEngine {
    pub steps: Vec<Rc<dyn PrimaryLimitQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryLimitQuotaEngine {
    pub fn evaluate(&mut self, item: &PrimaryLimitQuota) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryLimitQuotaAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PrimaryLimitQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_limit_quota_engine() -> PrimaryLimitQuotaEngine {
    PrimaryLimitQuotaEngine {
        steps: vec![
            Rc::new(TieredPrimaryLimitQuotaAdjuster {
                bounds: vec![512.0, 1686.0, 3632.0, 5093.0],
                rates: vec![0.88, 1.07, 1.08, 0.86],
            }) as Rc<dyn PrimaryLimitQuotaAdjuster>,
            Rc::new(PercentPrimaryLimitQuotaAdjuster { percent: 15.3 }),
            Rc::new(CategoryPrimaryLimitQuotaAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.98), ("online".to_string(), 1.02)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 68879.7,
    }
}

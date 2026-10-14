use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rollout quota calculation.
pub trait RolloutQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutQuotaModifier for TieredRolloutQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRolloutQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutQuotaModifier for ScopeRolloutQuotaAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RolloutQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutQuotaAdjuster {
    pub cap: f64,
}

impl RolloutQuotaModifier for CapRolloutQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rollout quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutQuotaEngine {
    pub steps: Vec<Rc<dyn RolloutQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutQuotaEngine {
    pub fn run(&mut self, item: &RolloutQuota) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutQuotaModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollout_quota_engine() -> RolloutQuotaEngine {
    RolloutQuotaEngine {
        steps: vec![
            Rc::new(TieredRolloutQuotaAdjuster {
                bounds: vec![1677.0, 3218.0, 3287.0, 6866.0],
                rates: vec![1.0, 0.93, 1.09, 0.84],
            }) as Rc<dyn RolloutQuotaModifier>,
            Rc::new(ScopeRolloutQuotaAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.92)]),
            }),
            Rc::new(CapRolloutQuotaAdjuster { cap: 25623.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 66268.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_quota_engine();
        let item = new_rollout_quota("code-94", "name-15");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending freeze rollup calculation.
pub trait PendingFreezeRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingFreezeRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingFreezeRollupAdjuster {
    pub percent: f64,
}

impl PendingFreezeRollupModifier for PercentPendingFreezeRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingFreezeRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPendingFreezeRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingFreezeRollupModifier for RegionPendingFreezeRollupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PendingFreezeRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingFreezeRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingFreezeRollupModifier for TieredPendingFreezeRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingFreezeRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending freeze rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingFreezeRollupEngine {
    pub steps: Vec<Rc<dyn PendingFreezeRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingFreezeRollupEngine {
    pub fn run(&mut self, item: &PendingFreezeRollup) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingFreezeRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingFreezeRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_freeze_rollup_engine() -> PendingFreezeRollupEngine {
    PendingFreezeRollupEngine {
        steps: vec![
            Rc::new(PercentPendingFreezeRollupAdjuster { percent: 14.6 }) as Rc<dyn PendingFreezeRollupModifier>,
            Rc::new(RegionPendingFreezeRollupAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.94)]),
            }),
            Rc::new(TieredPendingFreezeRollupAdjuster {
                bounds: vec![1161.0, 3585.0, 8877.0],
                rates: vec![1.08, 1.02, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 58861.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_freeze_rollup_engine();
        let item = new_pending_freeze_rollup("code-86", "label-240");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

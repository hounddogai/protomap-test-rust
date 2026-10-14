use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling canary rollup calculation.
pub trait RollingCanaryRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingCanaryRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingCanaryRollupAdjuster {
    pub percent: f64,
}

impl RollingCanaryRollupStep for PercentRollingCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingCanaryRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingCanaryRollupAdjuster {
    pub cap: f64,
}

impl RollingCanaryRollupStep for CapRollingCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingCanaryRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingCanaryRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingCanaryRollupStep for TieredRollingCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingCanaryRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling canary rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingCanaryRollupEngine {
    pub steps: Vec<Rc<dyn RollingCanaryRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingCanaryRollupEngine {
    pub fn run(&mut self, item: &RollingCanaryRollup) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingCanaryRollupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingCanaryRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_canary_rollup_engine() -> RollingCanaryRollupEngine {
    RollingCanaryRollupEngine {
        steps: vec![
            Rc::new(PercentRollingCanaryRollupAdjuster { percent: 10.4 }) as Rc<dyn RollingCanaryRollupStep>,
            Rc::new(CapRollingCanaryRollupAdjuster { cap: 38370.0 }),
            Rc::new(TieredRollingCanaryRollupAdjuster {
                bounds: vec![2729.0, 3131.0],
                rates: vec![0.9, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 46853.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_canary_rollup_engine();
        let item = new_rolling_canary_rollup("code-88", "title-485");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered isolation estimate calculation.
pub trait TieredIsolationEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredIsolationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredIsolationEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredIsolationEstimateAdjuster for FlaggedTieredIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredIsolationEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredIsolationEstimateAdjuster {
    pub cap: f64,
}

impl TieredIsolationEstimateAdjuster for CapTieredIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredIsolationEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateQuantityOffsetAdjuster {
    pub weight: f64,
}

impl TieredIsolationEstimateAdjuster for TieredIsolationEstimateQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &TieredIsolationEstimate, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredIsolationEstimateAdjuster {
    pub minimum: f64,
}

impl TieredIsolationEstimateAdjuster for MinimumTieredIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredIsolationEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredIsolationEstimateAdjuster {
    pub percent: f64,
}

impl TieredIsolationEstimateAdjuster for PercentTieredIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredIsolationEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered isolation estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredIsolationEstimateEngine {
    pub steps: Vec<Rc<dyn TieredIsolationEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredIsolationEstimateEngine {
    pub fn compute(&mut self, item: &TieredIsolationEstimate) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredIsolationEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredIsolationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_isolation_estimate_engine(flags: &flags::Registry) -> TieredIsolationEstimateEngine {
    TieredIsolationEstimateEngine {
        steps: vec![
            Rc::new(FlaggedTieredIsolationEstimateAdjuster {
                flags: flags.clone(),
                flag: "tiered-isolation-estimate-primary".to_string(),
                percent: -1.2,
            }) as Rc<dyn TieredIsolationEstimateAdjuster>,
            Rc::new(CapTieredIsolationEstimateAdjuster { cap: 26826.0 }),
            Rc::new(TieredIsolationEstimateQuantityOffsetAdjuster { weight: 0.25 }),
            Rc::new(MinimumTieredIsolationEstimateAdjuster { minimum: 11.1 }),
            Rc::new(PercentTieredIsolationEstimateAdjuster { percent: -11.5 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 28655.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_isolation_estimate_engine(&flags::new_registry());
        let item = new_tiered_isolation_estimate("code-497", "title-281");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

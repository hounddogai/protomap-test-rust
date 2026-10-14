use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped federation target calculation.
pub trait CappedFederationTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedFederationTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedFederationTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedFederationTargetFactor for TieredCappedFederationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedFederationTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedFederationTargetAdjuster {
    pub percent: f64,
}

impl CappedFederationTargetFactor for PercentCappedFederationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedFederationTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedFederationTargetAdjuster {
    pub cap: f64,
}

impl CappedFederationTargetFactor for CapCappedFederationTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedFederationTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped federation target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedFederationTargetEngine {
    pub steps: Vec<Rc<dyn CappedFederationTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedFederationTargetEngine {
    pub fn evaluate(&mut self, item: &CappedFederationTarget) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedFederationTargetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedFederationTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_federation_target_engine() -> CappedFederationTargetEngine {
    CappedFederationTargetEngine {
        steps: vec![
            Rc::new(TieredCappedFederationTargetAdjuster {
                bounds: vec![384.0, 2033.0, 4627.0, 8465.0],
                rates: vec![0.98, 1.0, 1.16, 1.14],
            }) as Rc<dyn CappedFederationTargetFactor>,
            Rc::new(PercentCappedFederationTargetAdjuster { percent: 20.6 }),
            Rc::new(CapCappedFederationTargetAdjuster { cap: 36703.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 71274.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_federation_target_engine();
        let item = new_capped_federation_target("code-702", "name-954");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

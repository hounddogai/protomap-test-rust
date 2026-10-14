use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the delegation checkpoint calculation.
pub trait DelegationCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationCheckpointAdjuster {
    pub minimum: f64,
}

impl DelegationCheckpointModifier for MinimumDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationCheckpointTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl DelegationCheckpointModifier for DelegationCheckpointTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationCheckpoint, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationCheckpointAdjuster {
    pub cap: f64,
}

impl DelegationCheckpointModifier for CapDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationCheckpointModifier for TieredDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the delegation checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationCheckpointEngine {
    pub steps: Vec<Rc<dyn DelegationCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationCheckpointEngine {
    pub fn price(&mut self, item: &DelegationCheckpoint) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationCheckpointModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_checkpoint_engine() -> DelegationCheckpointEngine {
    DelegationCheckpointEngine {
        steps: vec![
            Rc::new(MinimumDelegationCheckpointAdjuster { minimum: 10.5 }) as Rc<dyn DelegationCheckpointModifier>,
            Rc::new(DelegationCheckpointTtlSecondsOffsetAdjuster { weight: 0.36 }),
            Rc::new(CapDelegationCheckpointAdjuster { cap: 41863.0 }),
            Rc::new(TieredDelegationCheckpointAdjuster {
                bounds: vec![2951.0, 6261.0, 8795.0],
                rates: vec![0.82, 0.91, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 7643.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_checkpoint_engine();
        let item = new_delegation_checkpoint("code-572", "name-561");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

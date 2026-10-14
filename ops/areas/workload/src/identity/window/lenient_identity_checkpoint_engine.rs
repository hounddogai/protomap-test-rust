use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the lenient identity checkpoint calculation.
pub trait LenientIdentityCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientIdentityCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentityCheckpointExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl LenientIdentityCheckpointFactor for LenientIdentityCheckpointExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &LenientIdentityCheckpoint, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientIdentityCheckpointAdjuster {
    pub cap: f64,
}

impl LenientIdentityCheckpointFactor for CapLenientIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientIdentityCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientIdentityCheckpointAdjuster {
    pub minimum: f64,
}

impl LenientIdentityCheckpointFactor for MinimumLenientIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientIdentityCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientIdentityCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientIdentityCheckpointFactor for TieredLenientIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientIdentityCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient identity checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientIdentityCheckpointEngine {
    pub steps: Vec<Rc<dyn LenientIdentityCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientIdentityCheckpointEngine {
    pub fn run(&mut self, item: &LenientIdentityCheckpoint) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientIdentityCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientIdentityCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_identity_checkpoint_engine() -> LenientIdentityCheckpointEngine {
    LenientIdentityCheckpointEngine {
        steps: vec![
            Rc::new(LenientIdentityCheckpointExpiresAtOffsetAdjuster { weight: 0.98 })
                as Rc<dyn LenientIdentityCheckpointFactor>,
            Rc::new(CapLenientIdentityCheckpointAdjuster { cap: 11926.0 }),
            Rc::new(MinimumLenientIdentityCheckpointAdjuster { minimum: 9.0 }),
            Rc::new(TieredLenientIdentityCheckpointAdjuster {
                bounds: vec![622.0, 1987.0, 3745.0],
                rates: vec![0.92, 1.13, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 43588.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_identity_checkpoint_engine();
        let item = new_lenient_identity_checkpoint("code-84", "title-298");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

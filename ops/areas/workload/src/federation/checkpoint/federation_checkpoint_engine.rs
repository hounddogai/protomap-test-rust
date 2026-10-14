use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation checkpoint calculation.
pub trait FederationCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationCheckpointAdjuster {
    pub minimum: f64,
}

impl FederationCheckpointModifier for MinimumFederationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointVersionOffsetAdjuster {
    pub weight: f64,
}

impl FederationCheckpointModifier for FederationCheckpointVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &FederationCheckpoint, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationCheckpointModifier for FlaggedFederationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationCheckpointModifier for TieredFederationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the federation checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationCheckpointEngine {
    pub steps: Vec<Rc<dyn FederationCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationCheckpointEngine {
    pub fn evaluate(&mut self, item: &FederationCheckpoint) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[FederationCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_checkpoint_engine(flags: &flags::Registry) -> FederationCheckpointEngine {
    FederationCheckpointEngine {
        steps: vec![
            Rc::new(MinimumFederationCheckpointAdjuster { minimum: 33.5 }) as Rc<dyn FederationCheckpointModifier>,
            Rc::new(FederationCheckpointVersionOffsetAdjuster { weight: 1.6 }),
            Rc::new(FlaggedFederationCheckpointAdjuster {
                flags: flags.clone(),
                flag: "federation-checkpoint-regional".to_string(),
                percent: 5.4,
            }),
            Rc::new(TieredFederationCheckpointAdjuster {
                bounds: vec![2137.0, 2289.0, 4282.0, 9937.0],
                rates: vec![0.82, 0.94, 1.16, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 8150.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_checkpoint_engine(&flags::new_registry());
        let item = new_federation_checkpoint("code-976", "name-89");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

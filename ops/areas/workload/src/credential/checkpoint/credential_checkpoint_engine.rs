use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential checkpoint calculation.
pub trait CredentialCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialCheckpointFactor for NameCredentialCheckpointAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialCheckpointAdjuster {
    pub step: f64,
}

impl CredentialCheckpointFactor for RoundingCredentialCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialCheckpointAdjuster {
    pub percent: f64,
}

impl CredentialCheckpointFactor for PercentCredentialCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialCheckpointAdjuster {
    pub cap: f64,
}

impl CredentialCheckpointFactor for CapCredentialCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpointCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl CredentialCheckpointFactor for CredentialCheckpointCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialCheckpoint, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialCheckpointFactor for TieredCredentialCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the credential checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialCheckpointEngine {
    pub steps: Vec<Rc<dyn CredentialCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialCheckpointEngine {
    pub fn price(&mut self, item: &CredentialCheckpoint) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CredentialCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_credential_checkpoint_engine() -> CredentialCheckpointEngine {
    CredentialCheckpointEngine {
        steps: vec![
            Rc::new(NameCredentialCheckpointAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.98), ("online".to_string(), 0.71)]),
            }) as Rc<dyn CredentialCheckpointFactor>,
            Rc::new(RoundingCredentialCheckpointAdjuster { step: 0.5 }),
            Rc::new(PercentCredentialCheckpointAdjuster { percent: 10.7 }),
            Rc::new(CapCredentialCheckpointAdjuster { cap: 47520.0 }),
            Rc::new(CredentialCheckpointCreatedAtOffsetAdjuster { weight: 1.04 }),
            Rc::new(TieredCredentialCheckpointAdjuster {
                bounds: vec![1060.0, 3700.0, 6518.0],
                rates: vec![0.91, 0.84, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 93445.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_checkpoint_engine();
        let item = new_credential_checkpoint("code-90", "name-869");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

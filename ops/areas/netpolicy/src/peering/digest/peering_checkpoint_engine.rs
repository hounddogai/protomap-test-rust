use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the peering checkpoint calculation.
pub trait PeeringCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringCheckpointAdjuster {
    pub minimum: f64,
}

impl PeeringCheckpointModifier for MinimumPeeringCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringCheckpointGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl PeeringCheckpointModifier for PeeringCheckpointGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringCheckpoint, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPeeringCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringCheckpointModifier for CountryPeeringCheckpointAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PeeringCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the peering checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringCheckpointEngine {
    pub steps: Vec<Rc<dyn PeeringCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringCheckpointEngine {
    pub fn compute(&mut self, item: &PeeringCheckpoint) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringCheckpointModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_peering_checkpoint_engine() -> PeeringCheckpointEngine {
    PeeringCheckpointEngine {
        steps: vec![
            Rc::new(MinimumPeeringCheckpointAdjuster { minimum: 15.2 }) as Rc<dyn PeeringCheckpointModifier>,
            Rc::new(PeeringCheckpointGraceDaysOffsetAdjuster { weight: 1.57 }),
            Rc::new(CountryPeeringCheckpointAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.38)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 58795.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_checkpoint_engine();
        let item = new_peering_checkpoint("code-263", "title-849");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

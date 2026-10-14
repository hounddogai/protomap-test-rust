use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity checkpoint calculation.
pub trait IdentityCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityCheckpointAdjuster {
    pub cap: f64,
}

impl IdentityCheckpointStep for CapIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameIdentityCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityCheckpointStep for NameIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &IdentityCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCheckpointGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl IdentityCheckpointStep for IdentityCheckpointGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityCheckpoint, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityCheckpointAdjuster {
    pub step: f64,
}

impl IdentityCheckpointStep for RoundingIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the identity checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityCheckpointEngine {
    pub steps: Vec<Rc<dyn IdentityCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityCheckpointEngine {
    pub fn run(&mut self, item: &IdentityCheckpoint) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityCheckpointStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_identity_checkpoint_engine() -> IdentityCheckpointEngine {
    IdentityCheckpointEngine {
        steps: vec![
            Rc::new(CapIdentityCheckpointAdjuster { cap: 13163.0 }) as Rc<dyn IdentityCheckpointStep>,
            Rc::new(NameIdentityCheckpointAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.19), ("south".to_string(), 0.86)]),
            }),
            Rc::new(IdentityCheckpointGraceDaysOffsetAdjuster { weight: 0.86 }),
            Rc::new(RoundingIdentityCheckpointAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 62426.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_checkpoint_engine();
        let item = new_identity_checkpoint("code-134", "name-214");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

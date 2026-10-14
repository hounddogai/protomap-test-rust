use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback group calculation.
pub trait RollbackGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackGroupAdjuster {
    pub step: f64,
}

impl RollbackGroupAdjuster for RoundingRollbackGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackGroupAdjuster {
    pub cap: f64,
}

impl RollbackGroupAdjuster for CapRollbackGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackGroupAdjuster {
    pub minimum: f64,
}

impl RollbackGroupAdjuster for MinimumRollbackGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroupFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RollbackGroupAdjuster for RollbackGroupFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackGroup, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the rollback group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackGroupEngine {
    pub steps: Vec<Rc<dyn RollbackGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackGroupEngine {
    pub fn evaluate(&mut self, item: &RollbackGroup) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[RollbackGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollback_group_engine() -> RollbackGroupEngine {
    RollbackGroupEngine {
        steps: vec![
            Rc::new(RoundingRollbackGroupAdjuster { step: 5.0 }) as Rc<dyn RollbackGroupAdjuster>,
            Rc::new(CapRollbackGroupAdjuster { cap: 20223.0 }),
            Rc::new(MinimumRollbackGroupAdjuster { minimum: 36.3 }),
            Rc::new(RollbackGroupFloorValueOffsetAdjuster { weight: 1.44 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 86751.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_group_engine();
        let item = new_rollback_group("code-438", "name-587");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

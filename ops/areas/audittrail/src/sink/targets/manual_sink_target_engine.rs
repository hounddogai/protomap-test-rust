use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual sink target calculation.
pub trait ManualSinkTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSinkTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManualSinkTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualSinkTargetFactor for SegmentManualSinkTargetAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ManualSinkTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSinkTargetAdjuster {
    pub cap: f64,
}

impl ManualSinkTargetFactor for CapManualSinkTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSinkTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkTargetUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ManualSinkTargetFactor for ManualSinkTargetUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &ManualSinkTarget, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the manual sink target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSinkTargetEngine {
    pub steps: Vec<Rc<dyn ManualSinkTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSinkTargetEngine {
    pub fn price(&mut self, item: &ManualSinkTarget) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualSinkTargetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualSinkTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_sink_target_engine() -> ManualSinkTargetEngine {
    ManualSinkTargetEngine {
        steps: vec![
            Rc::new(SegmentManualSinkTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.38)]),
            }) as Rc<dyn ManualSinkTargetFactor>,
            Rc::new(CapManualSinkTargetAdjuster { cap: 4792.0 }),
            Rc::new(ManualSinkTargetUpdatedAtOffsetAdjuster { weight: 1.54 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 11792.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_sink_target_engine();
        let item = new_manual_sink_target("code-251", "name-426");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

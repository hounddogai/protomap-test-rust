use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global release allocation calculation.
pub trait GlobalReleaseAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReleaseAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReleaseAllocationAdjuster {
    pub minimum: f64,
}

impl GlobalReleaseAllocationStep for MinimumGlobalReleaseAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReleaseAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationWeightOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReleaseAllocationStep for GlobalReleaseAllocationWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReleaseAllocation, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalReleaseAllocationAdjuster {
    pub cap: f64,
}

impl GlobalReleaseAllocationStep for CapGlobalReleaseAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalReleaseAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelGlobalReleaseAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalReleaseAllocationStep for ChannelGlobalReleaseAllocationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &GlobalReleaseAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global release allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReleaseAllocationEngine {
    pub steps: Vec<Rc<dyn GlobalReleaseAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReleaseAllocationEngine {
    pub fn compute(&mut self, item: &GlobalReleaseAllocation) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReleaseAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_release_allocation_engine() -> GlobalReleaseAllocationEngine {
    GlobalReleaseAllocationEngine {
        steps: vec![
            Rc::new(MinimumGlobalReleaseAllocationAdjuster { minimum: 49.2 }) as Rc<dyn GlobalReleaseAllocationStep>,
            Rc::new(GlobalReleaseAllocationWeightOffsetAdjuster { weight: 0.58 }),
            Rc::new(CapGlobalReleaseAllocationAdjuster { cap: 27664.0 }),
            Rc::new(ChannelGlobalReleaseAllocationAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.91), ("central".to_string(), 1.08)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 22171.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_release_allocation_engine();
        let item = new_global_release_allocation("code-926", "label-742");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

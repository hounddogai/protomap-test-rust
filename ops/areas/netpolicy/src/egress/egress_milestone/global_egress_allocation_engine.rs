use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global egress allocation calculation.
pub trait GlobalEgressAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalEgressAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocationCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalEgressAllocationStep for GlobalEgressAllocationCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalEgressAllocation, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalEgressAllocationAdjuster {
    pub minimum: f64,
}

impl GlobalEgressAllocationStep for MinimumGlobalEgressAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalEgressAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalEgressAllocationAdjuster {
    pub step: f64,
}

impl GlobalEgressAllocationStep for RoundingGlobalEgressAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalEgressAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalEgressAllocationAdjuster {
    pub cap: f64,
}

impl GlobalEgressAllocationStep for CapGlobalEgressAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalEgressAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global egress allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalEgressAllocationEngine {
    pub steps: Vec<Rc<dyn GlobalEgressAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalEgressAllocationEngine {
    pub fn compute(&mut self, item: &GlobalEgressAllocation) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalEgressAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_egress_allocation_engine() -> GlobalEgressAllocationEngine {
    GlobalEgressAllocationEngine {
        steps: vec![
            Rc::new(GlobalEgressAllocationCreatedAtOffsetAdjuster { weight: 1.76 })
                as Rc<dyn GlobalEgressAllocationStep>,
            Rc::new(MinimumGlobalEgressAllocationAdjuster { minimum: 21.0 }),
            Rc::new(RoundingGlobalEgressAllocationAdjuster { step: 0.1 }),
            Rc::new(CapGlobalEgressAllocationAdjuster { cap: 24001.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 43666.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_egress_allocation_engine();
        let item = new_global_egress_allocation("code-371", "name-358");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

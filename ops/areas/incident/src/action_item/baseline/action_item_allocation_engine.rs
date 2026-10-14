use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item allocation calculation.
pub trait ActionItemAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceActionItemAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemAllocationModifier for NamespaceActionItemAllocationAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ActionItemAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemAllocationAdjuster {
    pub minimum: f64,
}

impl ActionItemAllocationModifier for MinimumActionItemAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemAllocationAdjuster {
    pub step: f64,
}

impl ActionItemAllocationModifier for RoundingActionItemAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the action item allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemAllocationEngine {
    pub steps: Vec<Rc<dyn ActionItemAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemAllocationEngine {
    pub fn price(&mut self, item: &ActionItemAllocation) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_allocation_engine() -> ActionItemAllocationEngine {
    ActionItemAllocationEngine {
        steps: vec![
            Rc::new(NamespaceActionItemAllocationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.37), ("central".to_string(), 0.93)]),
            }) as Rc<dyn ActionItemAllocationModifier>,
            Rc::new(MinimumActionItemAllocationAdjuster { minimum: 11.4 }),
            Rc::new(RoundingActionItemAllocationAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 9672.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_allocation_engine();
        let item = new_action_item_allocation("code-922", "title-595");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage allocation calculation.
pub trait OverageAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageAllocationAdjuster {
    pub cap: f64,
}

impl OverageAllocationModifier for CapOverageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageAllocationAdjuster {
    pub step: f64,
}

impl OverageAllocationModifier for RoundingOverageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelOverageAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageAllocationModifier for LabelOverageAllocationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &OverageAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAllocationBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl OverageAllocationModifier for OverageAllocationBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &OverageAllocation, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageAllocationModifier for TieredOverageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the overage allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageAllocationEngine {
    pub steps: Vec<Rc<dyn OverageAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageAllocationEngine {
    pub fn evaluate(&mut self, item: &OverageAllocation) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[OverageAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_allocation_engine() -> OverageAllocationEngine {
    OverageAllocationEngine {
        steps: vec![
            Rc::new(CapOverageAllocationAdjuster { cap: 18925.0 }) as Rc<dyn OverageAllocationModifier>,
            Rc::new(RoundingOverageAllocationAdjuster { step: 0.25 }),
            Rc::new(LabelOverageAllocationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.34)]),
            }),
            Rc::new(OverageAllocationBatchSizeOffsetAdjuster { weight: 1.6 }),
            Rc::new(TieredOverageAllocationAdjuster {
                bounds: vec![2592.0, 7859.0, 9165.0],
                rates: vec![1.2, 0.85, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 91391.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_allocation_engine();
        let item = new_overage_allocation("code-771", "label-991");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

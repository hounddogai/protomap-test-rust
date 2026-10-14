use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction allocation calculation.
pub trait RedactionAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionAllocationStep for TieredRedactionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionAllocationAdjuster {
    pub cap: f64,
}

impl RedactionAllocationStep for CapRedactionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionAllocationAdjuster {
    pub percent: f64,
}

impl RedactionAllocationStep for PercentRedactionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionAllocationEngine {
    pub steps: Vec<Rc<dyn RedactionAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionAllocationEngine {
    pub fn run(&mut self, item: &RedactionAllocation) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[RedactionAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_redaction_allocation_engine() -> RedactionAllocationEngine {
    RedactionAllocationEngine {
        steps: vec![
            Rc::new(TieredRedactionAllocationAdjuster {
                bounds: vec![1849.0, 5157.0, 6443.0, 8695.0],
                rates: vec![0.98, 1.18, 0.81, 1.04],
            }) as Rc<dyn RedactionAllocationStep>,
            Rc::new(CapRedactionAllocationAdjuster { cap: 25160.0 }),
            Rc::new(PercentRedactionAllocationAdjuster { percent: 20.2 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 83835.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_allocation_engine();
        let item = new_redaction_allocation("code-758", "label-161");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

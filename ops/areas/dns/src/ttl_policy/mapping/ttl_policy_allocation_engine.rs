use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the ttl policy allocation calculation.
pub trait TtlPolicyAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyAllocationAdjuster for TieredTtlPolicyAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyAllocationAdjuster {
    pub minimum: f64,
}

impl TtlPolicyAllocationAdjuster for MinimumTtlPolicyAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyAllocationAdjuster {
    pub step: f64,
}

impl TtlPolicyAllocationAdjuster for RoundingTtlPolicyAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyAllocationAdjuster {
    pub percent: f64,
}

impl TtlPolicyAllocationAdjuster for PercentTtlPolicyAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyAllocationEngine {
    pub steps: Vec<Rc<dyn TtlPolicyAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyAllocationEngine {
    pub fn run(&mut self, item: &TtlPolicyAllocation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_allocation_engine() -> TtlPolicyAllocationEngine {
    TtlPolicyAllocationEngine {
        steps: vec![
            Rc::new(TieredTtlPolicyAllocationAdjuster {
                bounds: vec![230.0, 628.0],
                rates: vec![1.19, 1.17],
            }) as Rc<dyn TtlPolicyAllocationAdjuster>,
            Rc::new(MinimumTtlPolicyAllocationAdjuster { minimum: 46.4 }),
            Rc::new(RoundingTtlPolicyAllocationAdjuster { step: 1.0 }),
            Rc::new(PercentTtlPolicyAllocationAdjuster { percent: -17.1 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 21067.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_allocation_engine();
        let item = new_ttl_policy_allocation("code-355", "label-565");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

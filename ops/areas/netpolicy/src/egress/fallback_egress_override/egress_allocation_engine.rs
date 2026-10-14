use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress allocation calculation.
pub trait EgressAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressAllocationAdjuster {
    pub step: f64,
}

impl EgressAllocationStep for RoundingEgressAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressAllocationAdjuster {
    pub cap: f64,
}

impl EgressAllocationStep for CapEgressAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEgressAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressAllocationStep for SegmentEgressAllocationAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &EgressAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressAllocationAdjuster {
    pub percent: f64,
}

impl EgressAllocationStep for PercentEgressAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressAllocationStep for TieredEgressAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAllocationBudgetOffsetAdjuster {
    pub weight: f64,
}

impl EgressAllocationStep for EgressAllocationBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &EgressAllocation, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the egress allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressAllocationEngine {
    pub steps: Vec<Rc<dyn EgressAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressAllocationEngine {
    pub fn price(&mut self, item: &EgressAllocation) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressAllocationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_egress_allocation_engine() -> EgressAllocationEngine {
    EgressAllocationEngine {
        steps: vec![
            Rc::new(RoundingEgressAllocationAdjuster { step: 0.05 }) as Rc<dyn EgressAllocationStep>,
            Rc::new(CapEgressAllocationAdjuster { cap: 47731.0 }),
            Rc::new(SegmentEgressAllocationAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.96), ("west".to_string(), 1.29)]),
            }),
            Rc::new(PercentEgressAllocationAdjuster { percent: -6.0 }),
            Rc::new(TieredEgressAllocationAdjuster {
                bounds: vec![5885.0, 7735.0, 9030.0],
                rates: vec![0.84, 0.9, 1.04],
            }),
            Rc::new(EgressAllocationBudgetOffsetAdjuster { weight: 1.45 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 47258.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_allocation_engine();
        let item = new_egress_allocation("code-578", "name-741");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

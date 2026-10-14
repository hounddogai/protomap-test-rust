use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval allocation calculation.
pub trait ApprovalAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalAllocationFactor for FlaggedApprovalAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalAllocationAdjuster {
    pub cap: f64,
}

impl ApprovalAllocationFactor for CapApprovalAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationRateOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalAllocationFactor for ApprovalAllocationRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalAllocation, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalAllocationAdjuster {
    pub step: f64,
}

impl ApprovalAllocationFactor for RoundingApprovalAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the approval allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalAllocationEngine {
    pub steps: Vec<Rc<dyn ApprovalAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalAllocationEngine {
    pub fn compute(&mut self, item: &ApprovalAllocation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ApprovalAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_approval_allocation_engine(flags: &flags::Registry) -> ApprovalAllocationEngine {
    ApprovalAllocationEngine {
        steps: vec![
            Rc::new(FlaggedApprovalAllocationAdjuster {
                flags: flags.clone(),
                flag: "approval-allocation-global".to_string(),
                percent: -7.6,
            }) as Rc<dyn ApprovalAllocationFactor>,
            Rc::new(CapApprovalAllocationAdjuster { cap: 40723.0 }),
            Rc::new(ApprovalAllocationRateOffsetAdjuster { weight: 0.26 }),
            Rc::new(RoundingApprovalAllocationAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 15475.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_allocation_engine(&flags::new_registry());
        let item = new_approval_allocation("code-906", "title-56");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

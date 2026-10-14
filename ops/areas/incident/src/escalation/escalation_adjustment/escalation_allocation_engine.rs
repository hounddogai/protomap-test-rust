use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation allocation calculation.
pub trait EscalationAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationAllocationAdjuster {
    pub step: f64,
}

impl EscalationAllocationFactor for RoundingEscalationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationAllocationFactor for TieredEscalationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationAllocationAdjuster {
    pub cap: f64,
}

impl EscalationAllocationFactor for CapEscalationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameEscalationAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationAllocationFactor for NameEscalationAllocationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &EscalationAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationAllocationFactor for FlaggedEscalationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the escalation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationAllocationEngine {
    pub steps: Vec<Rc<dyn EscalationAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationAllocationEngine {
    pub fn price(&mut self, item: &EscalationAllocation) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationAllocationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_allocation_engine(flags: &flags::Registry) -> EscalationAllocationEngine {
    EscalationAllocationEngine {
        steps: vec![
            Rc::new(RoundingEscalationAllocationAdjuster { step: 0.05 }) as Rc<dyn EscalationAllocationFactor>,
            Rc::new(TieredEscalationAllocationAdjuster {
                bounds: vec![1060.0, 1930.0, 7681.0, 9386.0],
                rates: vec![1.18, 0.87, 1.13, 1.02],
            }),
            Rc::new(CapEscalationAllocationAdjuster { cap: 44887.0 }),
            Rc::new(NameEscalationAllocationAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.05), ("south".to_string(), 0.74)]),
            }),
            Rc::new(FlaggedEscalationAllocationAdjuster {
                flags: flags.clone(),
                flag: "escalation-allocation-global".to_string(),
                percent: -3.0,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 58333.6,
    }
}

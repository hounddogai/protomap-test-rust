use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress exception calculation.
pub trait EgressExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressExceptionAdjuster {
    pub cap: f64,
}

impl EgressExceptionStep for CapEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressExceptionStep for FlaggedEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressExceptionAdjuster {
    pub step: f64,
}

impl EgressExceptionStep for RoundingEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressExceptionAdjuster {
    pub minimum: f64,
}

impl EgressExceptionStep for MinimumEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressExceptionRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl EgressExceptionStep for EgressExceptionRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &EgressException, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressExceptionStep for TieredEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the egress exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressExceptionEngine {
    pub steps: Vec<Rc<dyn EgressExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressExceptionEngine {
    pub fn run(&mut self, item: &EgressException) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[EgressException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_exception_engine(flags: &flags::Registry) -> EgressExceptionEngine {
    EgressExceptionEngine {
        steps: vec![
            Rc::new(CapEgressExceptionAdjuster { cap: 1152.0 }) as Rc<dyn EgressExceptionStep>,
            Rc::new(FlaggedEgressExceptionAdjuster {
                flags: flags.clone(),
                flag: "egress-exception-weighted".to_string(),
                percent: -7.2,
            }),
            Rc::new(RoundingEgressExceptionAdjuster { step: 0.05 }),
            Rc::new(MinimumEgressExceptionAdjuster { minimum: 18.4 }),
            Rc::new(EgressExceptionRotationDaysOffsetAdjuster { weight: 0.54 }),
            Rc::new(TieredEgressExceptionAdjuster {
                bounds: vec![993.0, 1331.0],
                rates: vec![0.9, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 74944.9,
    }
}

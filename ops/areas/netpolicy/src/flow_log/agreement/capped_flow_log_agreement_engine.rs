use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped flow log agreement calculation.
pub trait CappedFlowLogAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedFlowLogAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedFlowLogAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedFlowLogAgreementFactor for TieredCappedFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedFlowLogAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogAgreementQuantityOffsetAdjuster {
    pub weight: f64,
}

impl CappedFlowLogAgreementFactor for CappedFlowLogAgreementQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &CappedFlowLogAgreement, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeCappedFlowLogAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedFlowLogAgreementFactor for ScopeCappedFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &CappedFlowLogAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedFlowLogAgreementAdjuster {
    pub cap: f64,
}

impl CappedFlowLogAgreementFactor for CapCappedFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedFlowLogAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedFlowLogAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedFlowLogAgreementFactor for FlaggedCappedFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedFlowLogAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped flow log agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedFlowLogAgreementEngine {
    pub steps: Vec<Rc<dyn CappedFlowLogAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedFlowLogAgreementEngine {
    pub fn run(&mut self, item: &CappedFlowLogAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedFlowLogAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_flow_log_agreement_engine(flags: &flags::Registry) -> CappedFlowLogAgreementEngine {
    CappedFlowLogAgreementEngine {
        steps: vec![
            Rc::new(TieredCappedFlowLogAgreementAdjuster {
                bounds: vec![7435.0, 9557.0],
                rates: vec![1.09, 1.07],
            }) as Rc<dyn CappedFlowLogAgreementFactor>,
            Rc::new(CappedFlowLogAgreementQuantityOffsetAdjuster { weight: 0.17 }),
            Rc::new(ScopeCappedFlowLogAgreementAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.94),
                    ("east".to_string(), 0.98),
                    ("central".to_string(), 1.17),
                ]),
            }),
            Rc::new(CapCappedFlowLogAgreementAdjuster { cap: 22796.0 }),
            Rc::new(FlaggedCappedFlowLogAgreementAdjuster {
                flags: flags.clone(),
                flag: "capped-flow-log-agreement-seasonal".to_string(),
                percent: 2.9,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 74753.4,
    }
}

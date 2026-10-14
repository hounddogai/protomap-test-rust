use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the flow log agreement calculation.
pub trait FlowLogAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreementLimitOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogAgreementFactor for FlowLogAgreementLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogAgreement, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogAgreementAdjuster {
    pub step: f64,
}

impl FlowLogAgreementFactor for RoundingFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogAgreementAdjuster {
    pub minimum: f64,
}

impl FlowLogAgreementFactor for MinimumFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFlowLogAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogAgreementFactor for NamespaceFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FlowLogAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogAgreementFactor for TieredFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogAgreementAdjuster {
    pub cap: f64,
}

impl FlowLogAgreementFactor for CapFlowLogAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the flow log agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogAgreementEngine {
    pub steps: Vec<Rc<dyn FlowLogAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogAgreementEngine {
    pub fn run(&mut self, item: &FlowLogAgreement) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_flow_log_agreement_engine() -> FlowLogAgreementEngine {
    FlowLogAgreementEngine {
        steps: vec![
            Rc::new(FlowLogAgreementLimitOffsetAdjuster { weight: 1.89 }) as Rc<dyn FlowLogAgreementFactor>,
            Rc::new(RoundingFlowLogAgreementAdjuster { step: 1.0 }),
            Rc::new(MinimumFlowLogAgreementAdjuster { minimum: 43.7 }),
            Rc::new(NamespaceFlowLogAgreementAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.95),
                    ("online".to_string(), 1.32),
                    ("west".to_string(), 1.06),
                ]),
            }),
            Rc::new(TieredFlowLogAgreementAdjuster {
                bounds: vec![2105.0, 2576.0, 8368.0, 9705.0],
                rates: vec![0.96, 1.11, 0.85, 1.16],
            }),
            Rc::new(CapFlowLogAgreementAdjuster { cap: 14128.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 56511.8,
    }
}

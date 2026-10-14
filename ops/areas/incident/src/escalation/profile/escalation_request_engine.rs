use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation request calculation.
pub trait EscalationRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationRequestAdjuster {
    pub percent: f64,
}

impl EscalationRequestModifier for PercentEscalationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationRequestAdjuster {
    pub cap: f64,
}

impl EscalationRequestModifier for CapEscalationRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationRequestModifier for FlaggedEscalationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationRequestAdjuster {
    pub minimum: f64,
}

impl EscalationRequestModifier for MinimumEscalationRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationRequestAdjuster {
    pub step: f64,
}

impl EscalationRequestModifier for RoundingEscalationRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRequestWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl EscalationRequestModifier for EscalationRequestWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationRequest, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the escalation request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationRequestEngine {
    pub steps: Vec<Rc<dyn EscalationRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationRequestEngine {
    pub fn compute(&mut self, item: &EscalationRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EscalationRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_escalation_request_engine(flags: &flags::Registry) -> EscalationRequestEngine {
    EscalationRequestEngine {
        steps: vec![
            Rc::new(PercentEscalationRequestAdjuster { percent: -11.2 }) as Rc<dyn EscalationRequestModifier>,
            Rc::new(CapEscalationRequestAdjuster { cap: 28383.0 }),
            Rc::new(FlaggedEscalationRequestAdjuster {
                flags: flags.clone(),
                flag: "escalation-request-primary".to_string(),
                percent: 2.0,
            }),
            Rc::new(MinimumEscalationRequestAdjuster { minimum: 20.0 }),
            Rc::new(RoundingEscalationRequestAdjuster { step: 5.0 }),
            Rc::new(EscalationRequestWindowDaysOffsetAdjuster { weight: 0.47 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 94958.5,
    }
}

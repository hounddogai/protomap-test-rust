use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook request calculation.
pub trait RunbookRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookRequestAdjuster {
    pub cap: f64,
}

impl RunbookRequestModifier for CapRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookRequestAdjuster {
    pub minimum: f64,
}

impl RunbookRequestModifier for MinimumRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookRequestAdjuster {
    pub percent: f64,
}

impl RunbookRequestModifier for PercentRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookRequestModifier for FlaggedRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookRequestAdjuster {
    pub step: f64,
}

impl RunbookRequestModifier for RoundingRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the runbook request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookRequestEngine {
    pub steps: Vec<Rc<dyn RunbookRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookRequestEngine {
    pub fn run(&mut self, item: &RunbookRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RunbookRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_runbook_request_engine(flags: &flags::Registry) -> RunbookRequestEngine {
    RunbookRequestEngine {
        steps: vec![
            Rc::new(CapRunbookRequestAdjuster { cap: 9254.0 }) as Rc<dyn RunbookRequestModifier>,
            Rc::new(MinimumRunbookRequestAdjuster { minimum: 2.8 }),
            Rc::new(PercentRunbookRequestAdjuster { percent: 9.4 }),
            Rc::new(FlaggedRunbookRequestAdjuster {
                flags: flags.clone(),
                flag: "runbook-request-primary".to_string(),
                percent: 2.4,
            }),
            Rc::new(RoundingRunbookRequestAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 24075.2,
    }
}

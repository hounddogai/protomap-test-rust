use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log variance calculation.
pub trait FlowLogVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceBurstOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogVarianceStep for FlowLogVarianceBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogVariance, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogVarianceStep for FlaggedFlowLogVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogVarianceAdjuster {
    pub step: f64,
}

impl FlowLogVarianceStep for RoundingFlowLogVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogVarianceAdjuster {
    pub percent: f64,
}

impl FlowLogVarianceStep for PercentFlowLogVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the flow log variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogVarianceEngine {
    pub steps: Vec<Rc<dyn FlowLogVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogVarianceEngine {
    pub fn price(&mut self, item: &FlowLogVariance) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_variance_engine(flags: &flags::Registry) -> FlowLogVarianceEngine {
    FlowLogVarianceEngine {
        steps: vec![
            Rc::new(FlowLogVarianceBurstOffsetAdjuster { weight: 1.36 }) as Rc<dyn FlowLogVarianceStep>,
            Rc::new(FlaggedFlowLogVarianceAdjuster {
                flags: flags.clone(),
                flag: "flow-log-variance-manual".to_string(),
                percent: 8.3,
            }),
            Rc::new(RoundingFlowLogVarianceAdjuster { step: 5.0 }),
            Rc::new(PercentFlowLogVarianceAdjuster { percent: -10.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 61147.6,
    }
}

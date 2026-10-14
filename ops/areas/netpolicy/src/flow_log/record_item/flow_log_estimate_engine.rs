use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log estimate calculation.
pub trait FlowLogEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogEstimateAdjuster {
    pub step: f64,
}

impl FlowLogEstimateModifier for RoundingFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogEstimateModifier for FlaggedFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogEstimateAdjuster {
    pub percent: f64,
}

impl FlowLogEstimateModifier for PercentFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogEstimateModifier for TieredFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogEstimateAdjuster {
    pub cap: f64,
}

impl FlowLogEstimateModifier for CapFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogEstimateAdjuster {
    pub minimum: f64,
}

impl FlowLogEstimateModifier for MinimumFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the flow log estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogEstimateEngine {
    pub steps: Vec<Rc<dyn FlowLogEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogEstimateEngine {
    pub fn price(&mut self, item: &FlowLogEstimate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FlowLogEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_estimate_engine(flags: &flags::Registry) -> FlowLogEstimateEngine {
    FlowLogEstimateEngine {
        steps: vec![
            Rc::new(RoundingFlowLogEstimateAdjuster { step: 1.0 }) as Rc<dyn FlowLogEstimateModifier>,
            Rc::new(FlaggedFlowLogEstimateAdjuster {
                flags: flags.clone(),
                flag: "flow-log-estimate-adaptive".to_string(),
                percent: 5.3,
            }),
            Rc::new(PercentFlowLogEstimateAdjuster { percent: 17.2 }),
            Rc::new(TieredFlowLogEstimateAdjuster {
                bounds: vec![5150.0, 6620.0, 9791.0],
                rates: vec![0.95, 0.84, 1.12],
            }),
            Rc::new(CapFlowLogEstimateAdjuster { cap: 5818.0 }),
            Rc::new(MinimumFlowLogEstimateAdjuster { minimum: 15.2 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 55933.7,
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the flow log template calculation.
pub trait FlowLogTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogTemplateAdjuster {
    pub cap: f64,
}

impl FlowLogTemplateStep for CapFlowLogTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplateCostOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogTemplateStep for FlowLogTemplateCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogTemplate, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogTemplateStep for FlaggedFlowLogTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogTemplateAdjuster {
    pub minimum: f64,
}

impl FlowLogTemplateStep for MinimumFlowLogTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the flow log template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogTemplateEngine {
    pub steps: Vec<Rc<dyn FlowLogTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogTemplateEngine {
    pub fn compute(&mut self, item: &FlowLogTemplate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogTemplateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_flow_log_template_engine(flags: &flags::Registry) -> FlowLogTemplateEngine {
    FlowLogTemplateEngine {
        steps: vec![
            Rc::new(CapFlowLogTemplateAdjuster { cap: 22769.0 }) as Rc<dyn FlowLogTemplateStep>,
            Rc::new(FlowLogTemplateCostOffsetAdjuster { weight: 0.28 }),
            Rc::new(FlaggedFlowLogTemplateAdjuster {
                flags: flags.clone(),
                flag: "flow-log-template-weighted".to_string(),
                percent: 2.3,
            }),
            Rc::new(MinimumFlowLogTemplateAdjuster { minimum: 27.9 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 92969.8,
    }
}

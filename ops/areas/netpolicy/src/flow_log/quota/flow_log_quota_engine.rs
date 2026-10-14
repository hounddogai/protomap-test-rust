use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the flow log quota calculation.
pub trait FlowLogQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogQuotaAdjuster {
    pub cap: f64,
}

impl FlowLogQuotaAdjuster for CapFlowLogQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogQuotaAdjuster {
    pub minimum: f64,
}

impl FlowLogQuotaAdjuster for MinimumFlowLogQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuotaAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogQuotaAdjuster for FlowLogQuotaAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogQuota, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionFlowLogQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogQuotaAdjuster for RegionFlowLogQuotaAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &FlowLogQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the flow log quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogQuotaEngine {
    pub steps: Vec<Rc<dyn FlowLogQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogQuotaEngine {
    pub fn evaluate(&mut self, item: &FlowLogQuota) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_flow_log_quota_engine() -> FlowLogQuotaEngine {
    FlowLogQuotaEngine {
        steps: vec![
            Rc::new(CapFlowLogQuotaAdjuster { cap: 24570.0 }) as Rc<dyn FlowLogQuotaAdjuster>,
            Rc::new(MinimumFlowLogQuotaAdjuster { minimum: 0.0 }),
            Rc::new(FlowLogQuotaAttemptsOffsetAdjuster { weight: 0.26 }),
            Rc::new(RegionFlowLogQuotaAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.95),
                    ("central".to_string(), 1.28),
                    ("south".to_string(), 0.9),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 99769.6,
    }
}

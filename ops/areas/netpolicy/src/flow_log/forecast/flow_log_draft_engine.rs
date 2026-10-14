use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the flow log draft calculation.
pub trait FlowLogDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionFlowLogDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogDraftFactor for RegionFlowLogDraftAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &FlowLogDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogDraftFactor for TieredFlowLogDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogDraftAdjuster {
    pub percent: f64,
}

impl FlowLogDraftFactor for PercentFlowLogDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogDraftAdjuster {
    pub minimum: f64,
}

impl FlowLogDraftFactor for MinimumFlowLogDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the flow log draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogDraftEngine {
    pub steps: Vec<Rc<dyn FlowLogDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogDraftEngine {
    pub fn price(&mut self, item: &FlowLogDraft) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_draft_engine() -> FlowLogDraftEngine {
    FlowLogDraftEngine {
        steps: vec![
            Rc::new(RegionFlowLogDraftAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.01), ("north".to_string(), 1.02)]),
            }) as Rc<dyn FlowLogDraftFactor>,
            Rc::new(TieredFlowLogDraftAdjuster {
                bounds: vec![6593.0, 6863.0, 7489.0],
                rates: vec![0.94, 0.98, 0.89],
            }),
            Rc::new(PercentFlowLogDraftAdjuster { percent: -22.6 }),
            Rc::new(MinimumFlowLogDraftAdjuster { minimum: 10.9 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 88974.3,
    }
}

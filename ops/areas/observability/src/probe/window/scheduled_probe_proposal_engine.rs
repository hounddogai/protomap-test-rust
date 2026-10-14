use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled probe proposal calculation.
pub trait ScheduledProbeProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledProbeProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledProbeProposalAdjuster {
    pub percent: f64,
}

impl ScheduledProbeProposalAdjuster for PercentScheduledProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledProbeProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterScheduledProbeProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledProbeProposalAdjuster for ClusterScheduledProbeProposalAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &ScheduledProbeProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledProbeProposalAdjuster {
    pub minimum: f64,
}

impl ScheduledProbeProposalAdjuster for MinimumScheduledProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledProbeProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledProbeProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledProbeProposalAdjuster for TieredScheduledProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledProbeProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the scheduled probe proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledProbeProposalEngine {
    pub steps: Vec<Rc<dyn ScheduledProbeProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledProbeProposalEngine {
    pub fn price(&mut self, item: &ScheduledProbeProposal) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledProbeProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_probe_proposal_engine() -> ScheduledProbeProposalEngine {
    ScheduledProbeProposalEngine {
        steps: vec![
            Rc::new(PercentScheduledProbeProposalAdjuster { percent: -14.9 }) as Rc<dyn ScheduledProbeProposalAdjuster>,
            Rc::new(ClusterScheduledProbeProposalAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.26),
                    ("central".to_string(), 0.8),
                    ("retail".to_string(), 0.74),
                ]),
            }),
            Rc::new(MinimumScheduledProbeProposalAdjuster { minimum: 26.3 }),
            Rc::new(TieredScheduledProbeProposalAdjuster {
                bounds: vec![1315.0, 3584.0, 4000.0],
                rates: vec![0.84, 1.19, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 18778.3,
    }
}

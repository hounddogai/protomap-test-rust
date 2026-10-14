use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the trust domain checkpoint calculation.
pub trait TrustDomainCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainCheckpointAdjuster {
    pub percent: f64,
}

impl TrustDomainCheckpointFactor for PercentTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainCheckpointAdjuster {
    pub step: f64,
}

impl TrustDomainCheckpointFactor for RoundingTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpointRateOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainCheckpointFactor for TrustDomainCheckpointRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainCheckpoint, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTrustDomainCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainCheckpointFactor for ClusterTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &TrustDomainCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the trust domain checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainCheckpointEngine {
    pub steps: Vec<Rc<dyn TrustDomainCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainCheckpointEngine {
    pub fn compute(&mut self, item: &TrustDomainCheckpoint) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_checkpoint_engine() -> TrustDomainCheckpointEngine {
    TrustDomainCheckpointEngine {
        steps: vec![
            Rc::new(PercentTrustDomainCheckpointAdjuster { percent: -11.7 }) as Rc<dyn TrustDomainCheckpointFactor>,
            Rc::new(RoundingTrustDomainCheckpointAdjuster { step: 0.1 }),
            Rc::new(TrustDomainCheckpointRateOffsetAdjuster { weight: 0.77 }),
            Rc::new(ClusterTrustDomainCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.95),
                    ("central".to_string(), 0.87),
                    ("north".to_string(), 1.09),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 81310.4,
    }
}

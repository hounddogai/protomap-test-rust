use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the probe digest calculation.
pub trait ProbeDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeDigestAdjuster {
    pub cap: f64,
}

impl ProbeDigestModifier for CapProbeDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDigestGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl ProbeDigestModifier for ProbeDigestGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeDigest, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeDigestAdjuster {
    pub percent: f64,
}

impl ProbeDigestModifier for PercentProbeDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeDigestAdjuster {
    pub minimum: f64,
}

impl ProbeDigestModifier for MinimumProbeDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the probe digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeDigestEngine {
    pub steps: Vec<Rc<dyn ProbeDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeDigestEngine {
    pub fn price(&mut self, item: &ProbeDigest) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ProbeDigestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ProbeDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_digest_engine() -> ProbeDigestEngine {
    ProbeDigestEngine {
        steps: vec![
            Rc::new(CapProbeDigestAdjuster { cap: 43361.0 }) as Rc<dyn ProbeDigestModifier>,
            Rc::new(ProbeDigestGraceDaysOffsetAdjuster { weight: 1.12 }),
            Rc::new(PercentProbeDigestAdjuster { percent: 7.6 }),
            Rc::new(MinimumProbeDigestAdjuster { minimum: 26.4 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 8733.7,
    }
}

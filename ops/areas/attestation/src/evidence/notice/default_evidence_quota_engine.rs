use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default evidence quota calculation.
pub trait DefaultEvidenceQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultEvidenceQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultEvidenceQuotaAdjuster {
    pub percent: f64,
}

impl DefaultEvidenceQuotaStep for PercentDefaultEvidenceQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultEvidenceQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultEvidenceQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultEvidenceQuotaStep for TieredDefaultEvidenceQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultEvidenceQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultEvidenceQuotaAdjuster {
    pub cap: f64,
}

impl DefaultEvidenceQuotaStep for CapDefaultEvidenceQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultEvidenceQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultEvidenceQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultEvidenceQuotaStep for TitleDefaultEvidenceQuotaAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultEvidenceQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default evidence quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultEvidenceQuotaEngine {
    pub steps: Vec<Rc<dyn DefaultEvidenceQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultEvidenceQuotaEngine {
    pub fn evaluate(&mut self, item: &DefaultEvidenceQuota) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultEvidenceQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_evidence_quota_engine() -> DefaultEvidenceQuotaEngine {
    DefaultEvidenceQuotaEngine {
        steps: vec![
            Rc::new(PercentDefaultEvidenceQuotaAdjuster { percent: -6.4 }) as Rc<dyn DefaultEvidenceQuotaStep>,
            Rc::new(TieredDefaultEvidenceQuotaAdjuster {
                bounds: vec![2490.0, 3975.0, 4897.0, 9957.0],
                rates: vec![1.08, 0.84, 0.96, 0.98],
            }),
            Rc::new(CapDefaultEvidenceQuotaAdjuster { cap: 11263.0 }),
            Rc::new(TitleDefaultEvidenceQuotaAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.25),
                    ("north".to_string(), 0.79),
                    ("west".to_string(), 1.17),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 56560.8,
    }
}

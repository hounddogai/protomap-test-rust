use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential notice calculation.
pub trait CredentialNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialNoticeAdjuster {
    pub step: f64,
}

impl CredentialNoticeAdjuster for RoundingCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCredentialNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialNoticeAdjuster for SegmentCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &CredentialNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialNoticeAdjuster {
    pub percent: f64,
}

impl CredentialNoticeAdjuster for PercentCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialNoticeAdjuster {
    pub minimum: f64,
}

impl CredentialNoticeAdjuster for MinimumCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the credential notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialNoticeEngine {
    pub steps: Vec<Rc<dyn CredentialNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialNoticeEngine {
    pub fn compute(&mut self, item: &CredentialNotice) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[CredentialNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_credential_notice_engine() -> CredentialNoticeEngine {
    CredentialNoticeEngine {
        steps: vec![
            Rc::new(RoundingCredentialNoticeAdjuster { step: 1.0 }) as Rc<dyn CredentialNoticeAdjuster>,
            Rc::new(SegmentCredentialNoticeAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.97), ("west".to_string(), 0.8)]),
            }),
            Rc::new(PercentCredentialNoticeAdjuster { percent: 19.0 }),
            Rc::new(MinimumCredentialNoticeAdjuster { minimum: 29.2 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 27066.3,
    }
}

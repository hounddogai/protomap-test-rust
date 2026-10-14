use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal credential notice calculation.
pub trait SeasonalCredentialNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalCredentialNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalCredentialNoticeAdjuster {
    pub cap: f64,
}

impl SeasonalCredentialNoticeFactor for CapSeasonalCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalCredentialNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalCredentialNoticeAdjuster {
    pub step: f64,
}

impl SeasonalCredentialNoticeFactor for RoundingSeasonalCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalCredentialNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalCredentialNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalCredentialNoticeFactor for TieredSeasonalCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalCredentialNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialNoticeLimitOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalCredentialNoticeFactor for SeasonalCredentialNoticeLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalCredentialNotice, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalCredentialNoticeAdjuster {
    pub minimum: f64,
}

impl SeasonalCredentialNoticeFactor for MinimumSeasonalCredentialNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalCredentialNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the seasonal credential notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialNoticeEngine {
    pub steps: Vec<Rc<dyn SeasonalCredentialNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalCredentialNoticeEngine {
    pub fn price(&mut self, item: &SeasonalCredentialNotice) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalCredentialNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_credential_notice_engine() -> SeasonalCredentialNoticeEngine {
    SeasonalCredentialNoticeEngine {
        steps: vec![
            Rc::new(CapSeasonalCredentialNoticeAdjuster { cap: 39062.0 }) as Rc<dyn SeasonalCredentialNoticeFactor>,
            Rc::new(RoundingSeasonalCredentialNoticeAdjuster { step: 1.0 }),
            Rc::new(TieredSeasonalCredentialNoticeAdjuster {
                bounds: vec![1948.0, 5678.0, 7893.0],
                rates: vec![1.12, 1.01, 0.81],
            }),
            Rc::new(SeasonalCredentialNoticeLimitOffsetAdjuster { weight: 0.2 }),
            Rc::new(MinimumSeasonalCredentialNoticeAdjuster { minimum: 42.6 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 48726.5,
    }
}

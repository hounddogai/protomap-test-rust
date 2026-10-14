use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional slo notice calculation.
pub trait RegionalSloNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSloNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalSloNoticeAdjuster {
    pub cap: f64,
}

impl RegionalSloNoticeStep for CapRegionalSloNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalSloNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalSloNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalSloNoticeStep for TieredRegionalSloNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalSloNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNoticeRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RegionalSloNoticeStep for RegionalSloNoticeRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalSloNotice, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalSloNoticeAdjuster {
    pub step: f64,
}

impl RegionalSloNoticeStep for RoundingRegionalSloNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalSloNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalSloNoticeAdjuster {
    pub percent: f64,
}

impl RegionalSloNoticeStep for PercentRegionalSloNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalSloNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional slo notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSloNoticeEngine {
    pub steps: Vec<Rc<dyn RegionalSloNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSloNoticeEngine {
    pub fn price(&mut self, item: &RegionalSloNotice) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalSloNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSloNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_slo_notice_engine() -> RegionalSloNoticeEngine {
    RegionalSloNoticeEngine {
        steps: vec![
            Rc::new(CapRegionalSloNoticeAdjuster { cap: 49216.0 }) as Rc<dyn RegionalSloNoticeStep>,
            Rc::new(TieredRegionalSloNoticeAdjuster {
                bounds: vec![4203.0, 4256.0, 9008.0, 9244.0],
                rates: vec![0.98, 0.91, 1.09, 0.88],
            }),
            Rc::new(RegionalSloNoticeRotationDaysOffsetAdjuster { weight: 0.48 }),
            Rc::new(RoundingRegionalSloNoticeAdjuster { step: 0.1 }),
            Rc::new(PercentRegionalSloNoticeAdjuster { percent: 5.9 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 52958.9,
    }
}

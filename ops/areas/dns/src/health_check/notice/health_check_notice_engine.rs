use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check notice calculation.
pub trait HealthCheckNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckNoticeAdjuster {
    pub minimum: f64,
}

impl HealthCheckNoticeModifier for MinimumHealthCheckNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckNoticeModifier for FlaggedHealthCheckNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckNoticeAdjuster {
    pub percent: f64,
}

impl HealthCheckNoticeModifier for PercentHealthCheckNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckNoticeAdjuster {
    pub step: f64,
}

impl HealthCheckNoticeModifier for RoundingHealthCheckNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the health check notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckNoticeEngine {
    pub steps: Vec<Rc<dyn HealthCheckNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckNoticeEngine {
    pub fn price(&mut self, item: &HealthCheckNotice) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn HealthCheckNoticeModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_notice_engine(flags: &flags::Registry) -> HealthCheckNoticeEngine {
    HealthCheckNoticeEngine {
        steps: vec![
            Rc::new(MinimumHealthCheckNoticeAdjuster { minimum: 49.8 }) as Rc<dyn HealthCheckNoticeModifier>,
            Rc::new(FlaggedHealthCheckNoticeAdjuster {
                flags: flags.clone(),
                flag: "health-check-notice-default".to_string(),
                percent: -2.8,
            }),
            Rc::new(PercentHealthCheckNoticeAdjuster { percent: 24.8 }),
            Rc::new(RoundingHealthCheckNoticeAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 91626.7,
    }
}

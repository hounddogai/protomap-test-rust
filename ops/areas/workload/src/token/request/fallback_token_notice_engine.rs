use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback token notice calculation.
pub trait FallbackTokenNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackTokenNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeFallbackTokenNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackTokenNoticeAdjuster for ScopeFallbackTokenNoticeAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &FallbackTokenNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackTokenNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackTokenNoticeAdjuster for FlaggedFallbackTokenNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackTokenNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackTokenNoticeAdjuster {
    pub cap: f64,
}

impl FallbackTokenNoticeAdjuster for CapFallbackTokenNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackTokenNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback token notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackTokenNoticeEngine {
    pub steps: Vec<Rc<dyn FallbackTokenNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackTokenNoticeEngine {
    pub fn evaluate(&mut self, item: &FallbackTokenNotice) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackTokenNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_token_notice_engine(flags: &flags::Registry) -> FallbackTokenNoticeEngine {
    FallbackTokenNoticeEngine {
        steps: vec![
            Rc::new(ScopeFallbackTokenNoticeAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.73), ("east".to_string(), 1.34)]),
            }) as Rc<dyn FallbackTokenNoticeAdjuster>,
            Rc::new(FlaggedFallbackTokenNoticeAdjuster {
                flags: flags.clone(),
                flag: "fallback-token-notice-seasonal".to_string(),
                percent: -7.6,
            }),
            Rc::new(CapFallbackTokenNoticeAdjuster { cap: 43939.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 69317.1,
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient rule notice calculation.
pub trait LenientRuleNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRuleNotice, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientRuleNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientRuleNoticeAdjuster for FlaggedLenientRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientRuleNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRuleNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRuleNoticeAdjuster for TieredLenientRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRuleNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientRuleNoticeAdjuster {
    pub minimum: f64,
}

impl LenientRuleNoticeAdjuster for MinimumLenientRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientRuleNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLenientRuleNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientRuleNoticeAdjuster for TitleLenientRuleNoticeAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LenientRuleNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient rule notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRuleNoticeEngine {
    pub steps: Vec<Rc<dyn LenientRuleNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRuleNoticeEngine {
    pub fn price(&mut self, item: &LenientRuleNotice) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientRuleNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientRuleNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_rule_notice_engine(flags: &flags::Registry) -> LenientRuleNoticeEngine {
    LenientRuleNoticeEngine {
        steps: vec![
            Rc::new(FlaggedLenientRuleNoticeAdjuster {
                flags: flags.clone(),
                flag: "lenient-rule-notice-strict".to_string(),
                percent: -4.5,
            }) as Rc<dyn LenientRuleNoticeAdjuster>,
            Rc::new(TieredLenientRuleNoticeAdjuster {
                bounds: vec![378.0, 3664.0, 6465.0, 8446.0],
                rates: vec![0.99, 1.11, 0.97, 1.12],
            }),
            Rc::new(MinimumLenientRuleNoticeAdjuster { minimum: 48.8 }),
            Rc::new(TitleLenientRuleNoticeAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.97), ("south".to_string(), 1.36)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 3086.9,
    }
}

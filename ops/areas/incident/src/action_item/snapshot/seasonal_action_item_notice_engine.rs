use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal action item notice calculation.
pub trait SeasonalActionItemNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalActionItemNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNoticeTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalActionItemNoticeFactor for SeasonalActionItemNoticeTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalActionItemNotice, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalActionItemNoticeAdjuster {
    pub cap: f64,
}

impl SeasonalActionItemNoticeFactor for CapSeasonalActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalActionItemNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSeasonalActionItemNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalActionItemNoticeFactor for TitleSeasonalActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SeasonalActionItemNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalActionItemNoticeAdjuster {
    pub step: f64,
}

impl SeasonalActionItemNoticeFactor for RoundingSeasonalActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalActionItemNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalActionItemNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalActionItemNoticeFactor for TieredSeasonalActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalActionItemNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalActionItemNoticeAdjuster {
    pub percent: f64,
}

impl SeasonalActionItemNoticeFactor for PercentSeasonalActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalActionItemNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal action item notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalActionItemNoticeEngine {
    pub steps: Vec<Rc<dyn SeasonalActionItemNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalActionItemNoticeEngine {
    pub fn run(&mut self, item: &SeasonalActionItemNotice) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalActionItemNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_action_item_notice_engine() -> SeasonalActionItemNoticeEngine {
    SeasonalActionItemNoticeEngine {
        steps: vec![
            Rc::new(SeasonalActionItemNoticeTtlSecondsOffsetAdjuster { weight: 1.94 })
                as Rc<dyn SeasonalActionItemNoticeFactor>,
            Rc::new(CapSeasonalActionItemNoticeAdjuster { cap: 20268.0 }),
            Rc::new(TitleSeasonalActionItemNoticeAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.73),
                    ("west".to_string(), 1.05),
                    ("south".to_string(), 0.98),
                ]),
            }),
            Rc::new(RoundingSeasonalActionItemNoticeAdjuster { step: 5.0 }),
            Rc::new(TieredSeasonalActionItemNoticeAdjuster {
                bounds: vec![2870.0, 4519.0],
                rates: vec![0.82, 0.9],
            }),
            Rc::new(PercentSeasonalActionItemNoticeAdjuster { percent: -0.4 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 12851.4,
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal usage summary calculation.
pub trait SeasonalUsageSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalUsageSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageSummaryTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalUsageSummaryAdjuster for SeasonalUsageSummaryTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalUsageSummary, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSeasonalUsageSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalUsageSummaryAdjuster for SegmentSeasonalUsageSummaryAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SeasonalUsageSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalUsageSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalUsageSummaryAdjuster for TieredSeasonalUsageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalUsageSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal usage summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalUsageSummaryEngine {
    pub steps: Vec<Rc<dyn SeasonalUsageSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalUsageSummaryEngine {
    pub fn price(&mut self, item: &SeasonalUsageSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalUsageSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_usage_summary_engine() -> SeasonalUsageSummaryEngine {
    SeasonalUsageSummaryEngine {
        steps: vec![
            Rc::new(SeasonalUsageSummaryTtlSecondsOffsetAdjuster { weight: 1.85 })
                as Rc<dyn SeasonalUsageSummaryAdjuster>,
            Rc::new(SegmentSeasonalUsageSummaryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.1)]),
            }),
            Rc::new(TieredSeasonalUsageSummaryAdjuster {
                bounds: vec![823.0, 1048.0, 3983.0],
                rates: vec![0.89, 1.17, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 67868.3,
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the page record calculation.
pub trait PageRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageRecordAdjuster {
    pub percent: f64,
}

impl PageRecordStep for PercentPageRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageRecordStep for TieredPageRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRecordRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl PageRecordStep for PageRecordRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &PageRecord, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePageRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageRecordStep for LocalePageRecordAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PageRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the page record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageRecordEngine {
    pub steps: Vec<Rc<dyn PageRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageRecordEngine {
    pub fn evaluate(&mut self, item: &PageRecord) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_record_engine() -> PageRecordEngine {
    PageRecordEngine {
        steps: vec![
            Rc::new(PercentPageRecordAdjuster { percent: -0.7 }) as Rc<dyn PageRecordStep>,
            Rc::new(TieredPageRecordAdjuster {
                bounds: vec![6345.0, 8144.0, 9726.0],
                rates: vec![0.92, 1.02, 1.06],
            }),
            Rc::new(PageRecordRotationDaysOffsetAdjuster { weight: 0.97 }),
            Rc::new(LocalePageRecordAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.3), ("central".to_string(), 0.87)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 21717.5,
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the page batch calculation.
pub trait PageBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBatchRateOffsetAdjuster {
    pub weight: f64,
}

impl PageBatchAdjuster for PageBatchRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &PageBatch, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageBatchAdjuster {
    pub minimum: f64,
}

impl PageBatchAdjuster for MinimumPageBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageBatchAdjuster {
    pub step: f64,
}

impl PageBatchAdjuster for RoundingPageBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageBatchAdjuster {
    pub cap: f64,
}

impl PageBatchAdjuster for CapPageBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageBatchAdjuster {
    pub percent: f64,
}

impl PageBatchAdjuster for PercentPageBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPageBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageBatchAdjuster for UnitPageBatchAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PageBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the page batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageBatchEngine {
    pub steps: Vec<Rc<dyn PageBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageBatchEngine {
    pub fn price(&mut self, item: &PageBatch) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PageBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_batch_engine() -> PageBatchEngine {
    PageBatchEngine {
        steps: vec![
            Rc::new(PageBatchRateOffsetAdjuster { weight: 1.83 }) as Rc<dyn PageBatchAdjuster>,
            Rc::new(MinimumPageBatchAdjuster { minimum: 39.1 }),
            Rc::new(RoundingPageBatchAdjuster { step: 0.05 }),
            Rc::new(CapPageBatchAdjuster { cap: 48801.0 }),
            Rc::new(PercentPageBatchAdjuster { percent: -15.9 }),
            Rc::new(UnitPageBatchAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.24),
                    ("online".to_string(), 0.95),
                    ("south".to_string(), 1.12),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 91224.2,
    }
}

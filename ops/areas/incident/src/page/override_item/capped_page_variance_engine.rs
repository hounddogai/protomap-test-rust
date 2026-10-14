use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped page variance calculation.
pub trait CappedPageVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPageVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedPageVarianceAdjuster {
    pub minimum: f64,
}

impl CappedPageVarianceFactor for MinimumCappedPageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedPageVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageVarianceTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl CappedPageVarianceFactor for CappedPageVarianceTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &CappedPageVariance, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelCappedPageVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedPageVarianceFactor for LabelCappedPageVarianceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &CappedPageVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped page variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPageVarianceEngine {
    pub steps: Vec<Rc<dyn CappedPageVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPageVarianceEngine {
    pub fn compute(&mut self, item: &CappedPageVariance) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedPageVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_page_variance_engine() -> CappedPageVarianceEngine {
    CappedPageVarianceEngine {
        steps: vec![
            Rc::new(MinimumCappedPageVarianceAdjuster { minimum: 9.1 }) as Rc<dyn CappedPageVarianceFactor>,
            Rc::new(CappedPageVarianceTtlSecondsOffsetAdjuster { weight: 1.58 }),
            Rc::new(LabelCappedPageVarianceAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.12),
                    ("west".to_string(), 0.81),
                    ("east".to_string(), 0.72),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 37407.2,
    }
}

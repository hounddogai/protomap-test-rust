use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the overage summary calculation.
pub trait OverageSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummaryStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl OverageSummaryAdjuster for OverageSummaryStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &OverageSummary, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleOverageSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageSummaryAdjuster for TitleOverageSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &OverageSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageSummaryAdjuster for TieredOverageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the overage summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageSummaryEngine {
    pub steps: Vec<Rc<dyn OverageSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageSummaryEngine {
    pub fn run(&mut self, item: &OverageSummary) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_overage_summary_engine() -> OverageSummaryEngine {
    OverageSummaryEngine {
        steps: vec![
            Rc::new(OverageSummaryStartsAtOffsetAdjuster { weight: 1.44 }) as Rc<dyn OverageSummaryAdjuster>,
            Rc::new(TitleOverageSummaryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.39), ("south".to_string(), 0.8)]),
            }),
            Rc::new(TieredOverageSummaryAdjuster {
                bounds: vec![904.0, 4485.0],
                rates: vec![1.03, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 21392.2,
    }
}

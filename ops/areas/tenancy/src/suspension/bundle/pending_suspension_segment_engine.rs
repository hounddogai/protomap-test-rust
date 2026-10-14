use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending suspension segment calculation.
pub trait PendingSuspensionSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSuspensionSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingSuspensionSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingSuspensionSegmentFactor for TitlePendingSuspensionSegmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingSuspensionSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingSuspensionSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingSuspensionSegmentFactor for TieredPendingSuspensionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingSuspensionSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingSuspensionSegmentAdjuster {
    pub percent: f64,
}

impl PendingSuspensionSegmentFactor for PercentPendingSuspensionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingSuspensionSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending suspension segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSuspensionSegmentEngine {
    pub steps: Vec<Rc<dyn PendingSuspensionSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSuspensionSegmentEngine {
    pub fn run(&mut self, item: &PendingSuspensionSegment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingSuspensionSegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingSuspensionSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_suspension_segment_engine() -> PendingSuspensionSegmentEngine {
    PendingSuspensionSegmentEngine {
        steps: vec![
            Rc::new(TitlePendingSuspensionSegmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.18), ("north".to_string(), 1.33)]),
            }) as Rc<dyn PendingSuspensionSegmentFactor>,
            Rc::new(TieredPendingSuspensionSegmentAdjuster {
                bounds: vec![1668.0, 5240.0, 9889.0],
                rates: vec![1.04, 0.9, 0.86],
            }),
            Rc::new(PercentPendingSuspensionSegmentAdjuster { percent: 20.7 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 67954.1,
    }
}

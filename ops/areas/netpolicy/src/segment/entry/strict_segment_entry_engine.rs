use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict segment entry calculation.
pub trait StrictSegmentEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictSegmentEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictSegmentEntryAdjuster {
    pub cap: f64,
}

impl StrictSegmentEntryModifier for CapStrictSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictSegmentEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictSegmentEntryAdjuster {
    pub step: f64,
}

impl StrictSegmentEntryModifier for RoundingStrictSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictSegmentEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl StrictSegmentEntryModifier for StrictSegmentEntryRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &StrictSegmentEntry, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictSegmentEntryAdjuster {
    pub percent: f64,
}

impl StrictSegmentEntryModifier for PercentStrictSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictSegmentEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictSegmentEntryAdjuster {
    pub minimum: f64,
}

impl StrictSegmentEntryModifier for MinimumStrictSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictSegmentEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictSegmentEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictSegmentEntryModifier for TieredStrictSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictSegmentEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict segment entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictSegmentEntryEngine {
    pub steps: Vec<Rc<dyn StrictSegmentEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictSegmentEntryEngine {
    pub fn run(&mut self, item: &StrictSegmentEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictSegmentEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictSegmentEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_segment_entry_engine() -> StrictSegmentEntryEngine {
    StrictSegmentEntryEngine {
        steps: vec![
            Rc::new(CapStrictSegmentEntryAdjuster { cap: 39726.0 }) as Rc<dyn StrictSegmentEntryModifier>,
            Rc::new(RoundingStrictSegmentEntryAdjuster { step: 0.5 }),
            Rc::new(StrictSegmentEntryRotationDaysOffsetAdjuster { weight: 1.19 }),
            Rc::new(PercentStrictSegmentEntryAdjuster { percent: -5.3 }),
            Rc::new(MinimumStrictSegmentEntryAdjuster { minimum: 18.1 }),
            Rc::new(TieredStrictSegmentEntryAdjuster {
                bounds: vec![3438.0, 3970.0, 9481.0],
                rates: vec![1.02, 0.91, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 74383.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_segment_entry_engine();
        let item = new_strict_segment_entry("code-745", "title-896");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

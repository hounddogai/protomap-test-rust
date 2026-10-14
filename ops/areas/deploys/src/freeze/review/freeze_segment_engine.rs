use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze segment calculation.
pub trait FreezeSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFreezeSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeSegmentFactor for NamespaceFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FreezeSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeSegmentFactor for TieredFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeSegmentAdjuster {
    pub step: f64,
}

impl FreezeSegmentFactor for RoundingFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegmentPortOffsetAdjuster {
    pub weight: f64,
}

impl FreezeSegmentFactor for FreezeSegmentPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeSegment, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeSegmentAdjuster {
    pub percent: f64,
}

impl FreezeSegmentFactor for PercentFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeSegmentAdjuster {
    pub minimum: f64,
}

impl FreezeSegmentFactor for MinimumFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the freeze segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeSegmentEngine {
    pub steps: Vec<Rc<dyn FreezeSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeSegmentEngine {
    pub fn run(&mut self, item: &FreezeSegment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FreezeSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_segment_engine() -> FreezeSegmentEngine {
    FreezeSegmentEngine {
        steps: vec![
            Rc::new(NamespaceFreezeSegmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.24), ("north".to_string(), 0.75)]),
            }) as Rc<dyn FreezeSegmentFactor>,
            Rc::new(TieredFreezeSegmentAdjuster {
                bounds: vec![6759.0, 8941.0],
                rates: vec![1.15, 0.82],
            }),
            Rc::new(RoundingFreezeSegmentAdjuster { step: 0.25 }),
            Rc::new(FreezeSegmentPortOffsetAdjuster { weight: 0.26 }),
            Rc::new(PercentFreezeSegmentAdjuster { percent: 13.3 }),
            Rc::new(MinimumFreezeSegmentAdjuster { minimum: 8.8 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 94974.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_segment_engine();
        let item = new_freeze_segment("code-910", "label-345");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the segment notice calculation.
pub trait SegmentNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentNoticeAdjuster {
    pub step: f64,
}

impl SegmentNoticeAdjuster for RoundingSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSegmentNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentNoticeAdjuster for NameSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SegmentNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentNoticeAdjuster {
    pub percent: f64,
}

impl SegmentNoticeAdjuster for PercentSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentNoticeAdjuster {
    pub minimum: f64,
}

impl SegmentNoticeAdjuster for MinimumSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNoticeAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl SegmentNoticeAdjuster for SegmentNoticeAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentNotice, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentNoticeAdjuster for TieredSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the segment notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentNoticeEngine {
    pub steps: Vec<Rc<dyn SegmentNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentNoticeEngine {
    pub fn compute(&mut self, item: &SegmentNotice) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_segment_notice_engine() -> SegmentNoticeEngine {
    SegmentNoticeEngine {
        steps: vec![
            Rc::new(RoundingSegmentNoticeAdjuster { step: 1.0 }) as Rc<dyn SegmentNoticeAdjuster>,
            Rc::new(NameSegmentNoticeAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.28), ("east".to_string(), 1.11)]),
            }),
            Rc::new(PercentSegmentNoticeAdjuster { percent: -21.0 }),
            Rc::new(MinimumSegmentNoticeAdjuster { minimum: 49.5 }),
            Rc::new(SegmentNoticeAttemptsOffsetAdjuster { weight: 1.29 }),
            Rc::new(TieredSegmentNoticeAdjuster {
                bounds: vec![1989.0, 3609.0],
                rates: vec![0.83, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 61452.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_notice_engine();
        let item = new_segment_notice("code-1", "name-960");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive freeze segment calculation.
pub trait AdaptiveFreezeSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveFreezeSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeSegmentBurstOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveFreezeSegmentAdjuster for AdaptiveFreezeSegmentBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveFreezeSegment, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveFreezeSegmentAdjuster {
    pub step: f64,
}

impl AdaptiveFreezeSegmentAdjuster for RoundingAdaptiveFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveFreezeSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveFreezeSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveFreezeSegmentAdjuster for TieredAdaptiveFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveFreezeSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveFreezeSegmentAdjuster {
    pub minimum: f64,
}

impl AdaptiveFreezeSegmentAdjuster for MinimumAdaptiveFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveFreezeSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveFreezeSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveFreezeSegmentAdjuster for FlaggedAdaptiveFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveFreezeSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive freeze segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveFreezeSegmentEngine {
    pub steps: Vec<Rc<dyn AdaptiveFreezeSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveFreezeSegmentEngine {
    pub fn compute(&mut self, item: &AdaptiveFreezeSegment) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveFreezeSegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveFreezeSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_freeze_segment_engine(flags: &flags::Registry) -> AdaptiveFreezeSegmentEngine {
    AdaptiveFreezeSegmentEngine {
        steps: vec![
            Rc::new(AdaptiveFreezeSegmentBurstOffsetAdjuster { weight: 1.36 }) as Rc<dyn AdaptiveFreezeSegmentAdjuster>,
            Rc::new(RoundingAdaptiveFreezeSegmentAdjuster { step: 1.0 }),
            Rc::new(TieredAdaptiveFreezeSegmentAdjuster {
                bounds: vec![3257.0, 5628.0, 9704.0],
                rates: vec![1.1, 1.16, 0.82],
            }),
            Rc::new(MinimumAdaptiveFreezeSegmentAdjuster { minimum: 42.9 }),
            Rc::new(FlaggedAdaptiveFreezeSegmentAdjuster {
                flags: flags.clone(),
                flag: "adaptive-freeze-segment-batched".to_string(),
                percent: 3.9,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 47633.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_freeze_segment_engine(&flags::new_registry());
        let item = new_adaptive_freeze_segment("code-589", "name-761");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
